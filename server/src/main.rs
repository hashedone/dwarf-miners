use anyhow::Error;
use futures::pin_mut;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use tokio;
use tokio::net::TcpListener;

mod protocol;

use protocol::{Request, Response};

#[derive(thiserror::Error, Debug)]
enum ServerError {
    #[error("No client connected")]
    ClientMissing,
    #[error("Unexpected message from client: {0:?}")]
    UnexpectedMessage(Request),
    #[error("Connection closed")]
    ConnectionClosed,
}

async fn handle_client(
    stream: impl Stream<Item = Request>,
    sink: impl Sink<Response, Error = impl std::error::Error + Sync + Send + 'static>,
) -> Result<(), Error> {
    pin_mut!(stream);
    pin_mut!(sink);

    #[allow(unreachable_patterns)]
    match stream.next().await.ok_or(ServerError::ConnectionClosed)? {
        Request::Greeting => (),
        msg => return Err(ServerError::UnexpectedMessage(msg).into()),
    }

    sink.send(Response::Wellcome).await.map_err(Error::from)
}

async fn serve() -> Result<(), Error> {
    let mut listener = TcpListener::bind("127.0.0.1:3030").await?;
    let mut connections = listener.incoming();

    let client = connections
        .next()
        .await
        .ok_or(ServerError::ClientMissing)??;
    let (sink, stream) = protocol::make_codec(client).split();
    handle_client(stream, sink).await
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        println!("Unhandled error while serving game: {:?}", err);
    }
}
