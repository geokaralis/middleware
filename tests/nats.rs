use async_nats::Client;
use futures::stream::StreamExt;
use testcontainers::core::WaitFor;
use testcontainers::runners::AsyncRunner;
use testcontainers::GenericImage;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task;
use tracing::error;

async fn handle_client(
    listener: TcpListener,
    mut shutdown_rx: mpsc::Receiver<()>,
    nats_client: Client,
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
                    nats_client.publish("ecr.pos", "world".into()).await?;

                    let mut pos_subscriber = nats_client.subscribe("pos.ecr".to_string()).await?;

                    if let Some(pos_message) = pos_subscriber.next().await {
                      socket.write_all(pos_message.payload.as_ref()).await?;
                  }
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

async fn launch_server(
    nats_url: String,
) -> Result<
    (
        task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>,
        mpsc::Sender<()>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);
    let listener = TcpListener::bind("localhost:4000").await?;
    let nats_client = async_nats::connect(nats_url).await?;

    let server_task =
        tokio::spawn(async move { handle_client(listener, shutdown_rx, nats_client).await });

    Ok((server_task, shutdown_tx))
}

#[tokio::test]
async fn test_tcp_server_with_nats() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let image = GenericImage::new("nats", "latest")
        .with_exposed_port(4222)
        .with_wait_for(WaitFor::message_on_stderr(
            "Listening for client connections on 0.0.0.0:4222",
        ))
        .with_wait_for(WaitFor::message_on_stderr("Server is ready"));
    let container = image.start().await;

    let host = container.get_host().await;
    let host_port = container.get_host_port_ipv4(4222).await;
    let url = format!("{host}:{host_port}");

    let nats_client = async_nats::ConnectOptions::default()
        .connect(url.clone())
        .await?;

    let (server, shutdown_tx) = launch_server(url.clone()).await?;

    let mut pos_subscriber = nats_client.subscribe("ecr.pos".to_string()).await?;

    let mut ecr_stream = TcpStream::connect("localhost:4000").await?;
    let message = "hello";

    ecr_stream.write_all(message.as_bytes()).await?;

    let pos_message = pos_subscriber.next().await.unwrap();
    assert_eq!(pos_message.payload.as_ref(), b"world");

    nats_client.publish("pos.ecr", "back to ecr".into()).await?;

    let mut buf = [0; 1024];
    let n = ecr_stream.read(&mut buf).await?;
    let response = std::str::from_utf8(&buf[..n])?;
    println!("{}", response);
    assert_eq!(response, "back to ecr");

    shutdown_tx.send(()).await?;
    server.await??;

    Ok(())
}

// tokio::spawn(async move {
//     let mut buf = [0; 4 * 1024];

//     let n = socket.read(&mut buf).await.unwrap();

//     socket.write_all(&buf[0..n]).await.unwrap();

//     let data = str::from_utf8(&buf[0..n]).unwrap();
//     info!(data);

//     socket.shutdown().await.unwrap();
// });
