use bytes::Bytes;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

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

/// Takes tokio `AsyncRead + AsyncWrite` object, and returns `Stream + Sink` object for operating
/// directly on [Request](request/enum.Request.html) and [Response](response/enum.Response.html) types
pub fn make_codec<Transport>(
    transport: Transport,
) -> impl Stream<Item = Result<Request, CodecError>> + Sink<Response>
where
    Transport: AsyncWrite + AsyncRead,
{
    let transport = Framed::new(transport, LengthDelimitedCodec::new());
    let transport = transport.map(|bytes| -> Result<Request, CodecError> {
        Ok(rmp_serde::decode::from_read_ref(&bytes?)?)
    });

    async fn encode_response(response: Response) -> Result<Bytes, CodecError> {
        let data = rmp_serde::encode::to_vec_named(&response)?;
        Ok(data.into())
    }

    transport
        .sink_map_err(CodecError::from)
        .with(encode_response)
}

