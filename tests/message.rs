use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task;
use tracing::error;

#[tokio::test]
async fn test_tcp_server_hello_world() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (server, shutdown_tx) = launch_server().await?;

    let mut stream = TcpStream::connect("localhost:4000").await?;
    let message = "hello";

    stream.write_all(message.as_bytes()).await?;

    let mut buf = [0; 1024];
    let n = stream.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;

    assert_eq!(response, "world");

    shutdown_tx.send(()).await?;

    server.await??;

    Ok(())
}

async fn handle_client(
    listener: TcpListener,
    mut shutdown_rx: mpsc::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        tokio::select! {
            Ok((mut socket, _)) = listener.accept() => {
                let mut buf = [0; 1024];

                let n = socket.read(&mut buf).await?;
                if n == 0 {
                    continue;
                }
                let message = std::str::from_utf8(&buf[..n])?;

                if message == "hello" {
                    socket.write_all("world".as_bytes()).await?;
                } else {
                    error!("received unexpected message: {}", message);
                }
            },
            _ = shutdown_rx.recv() => {
                break;
            }
        }
    }
    Ok(())
}

async fn launch_server() -> Result<
    (
        task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
        mpsc::Sender<()>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
    let listener = TcpListener::bind("localhost:4000").await?;

    let server_task = tokio::spawn(async move { handle_client(listener, shutdown_rx).await });

    Ok((server_task, shutdown_tx))
}
