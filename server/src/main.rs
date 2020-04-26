use anyhow::Error;
use futures::stream::StreamExt;
use tokio;
use tokio::net::TcpListener;

mod protocol;

#[derive(thiserror::Error, Debug)]
enum ServerError {
    #[error("No client connected")]
    ClientMissing,
}

async fn serve() -> Result<(), Error> {
    let mut listener = TcpListener::bind("127.0.0.1:3030").await?;
    let mut connections = listener.incoming();

    let client = connections
        .next()
        .await
        .ok_or(ServerError::ClientMissing)??;
    let (stream, sink) = protocol::make_codec(client).split();

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = serve().await {
        println!("Unhandled error while serving game: {:?}", err);
    }
}
