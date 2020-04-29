use bytes::Bytes;
use futures::ready;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use pin_project::pin_project;
use std::fmt::Display;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{error, warn};

#[derive(thiserror::Error, Debug)]
pub enum CodecError {
    #[error("Message length invalid: {0}")]
    Length(#[from] io::Error),
    #[error("MsgPack mallformed: {0}")]
    MsgPackDecode(#[from] rmp_serde::decode::Error),
    #[error("MsgPack encoding error: {0}")]
    MsgPackEncode(#[from] rmp_serde::encode::Error),
}

pub mod request;
pub mod response;

pub use request::Request;
pub use response::Response;

/// Stream that forwards elements from parent while they are `Ok`, and when `Err` occurs it is
/// logged and `None` is returned so stream is ended
#[pin_project]
struct EnsureOkStream<Parent>(#[pin] Parent);

impl<Parent, Item, Error> Stream for EnsureOkStream<Parent>
where
    Parent: Stream<Item = Result<Item, Error>>,
    Error: Display,
{
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Item>> {
        let this = self.project();
        let result = match ready!(this.0.poll_next(cx)) {
            Some(Ok(item)) => Some(item),
            Some(Err(err)) => {
                error!(%err, "Invalid frame");
                None
            }
            None => None,
        };

        Poll::Ready(result)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

/// Takes tokio `AsyncRead + AsyncWrite` object, and returns `Stream + Sink` object for operating
/// directly on [Request](request/enum.Request.html) and [Response](response/enum.Response.html) types
pub fn make_codec<Transport>(
    transport: Transport,
) -> (
    impl Sink<Response, Error = CodecError>,
    impl Stream<Item = Request>,
)
where
    Transport: AsyncWrite + AsyncRead,
{
    let (sink, stream) = Framed::new(transport, LengthDelimitedCodec::new()).split();
    let stream = EnsureOkStream(stream);
    let stream = stream.filter_map(|bytes| async move {
        rmp_serde::decode::from_read_ref(&bytes)
            .map_err(|err| warn!(%err, msg = ?bytes, "Error while decoding message"))
            .ok()
    });

    async fn encode_response(response: Response) -> Result<Bytes, CodecError> {
        let data = rmp_serde::encode::to_vec_named(&response).map_err(|err| {
            warn!(?err, msg = ?response, "Error while encoding message");
            err
        })?;
        Ok(data.into())
    }

    let sink = sink
        .sink_map_err(|err| {
            warn!(%err, "Error while sending message");
            CodecError::from(err)
        })
        .with(encode_response);

    (sink, stream)
}
