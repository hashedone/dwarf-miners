use anyhow::Error;
use futures::pin_mut;
use futures::sink::{Sink, SinkExt};
use futures::stream::{Stream, StreamExt};
use tokio::net::TcpListener;
use tracing::{error, info, span, warn, Level};

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
    pin_mut!(sink, stream);

    #[allow(unreachable_patterns)]
    match stream.next().await.ok_or(ServerError::ConnectionClosed)? {
        Request::Greeting => (),
        msg => {
            warn!(?msg, "Unexpected message");
            return Err(ServerError::UnexpectedMessage(msg).into());
        }
    }

    sink.send(Response::Wellcome).await.map_err(Error::from)
}

#[tracing::instrument]
async fn serve() -> Result<(), Error> {
    info!("Binding server");
    let mut listener = TcpListener::bind("127.0.0.1:3030").await?;
    info!(addr = "127.0.0.1", port = 3030, "Server listening");

    let mut connections = listener.incoming();
    let client = connections
        .next()
        .await
        .ok_or(ServerError::ClientMissing)??;

    let span = span!(Level::TRACE, "connection");
    let _ = span.enter();

    info!("Client connected: {:?}", client.peer_addr());

    let (sink, stream) = protocol::make_codec(client);
    handle_client(stream, sink).await
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .init();
    info!("Logging initialized");

    if let Err(err) = serve().await {
        error!(%err, "Unhandled error while serving game");
    }
}
