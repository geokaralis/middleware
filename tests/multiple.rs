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
                let mut buf = vec![0; 1024];

                let n = socket.read(&mut buf).await?;
                if n == 0 {
                    continue;
                }

                let message = std::str::from_utf8(&buf[..n])?;
                println!("received message: {}", message);

                // message format "tid1:message"
                let parts: Vec<&str> = message.splitn(2, ':').collect();
                if parts.len() != 2 {
                    error!("invalid message format: {}", message);
                    continue;
                }
                let tid = parts[0].to_string();
                let actual_message = parts[1].to_string();

                let nats_channel = format!("ecr.pos/{}", tid);
                println!("publishing to nats channel: {}", nats_channel);
                nats_client.publish(nats_channel.clone(), actual_message.into()).await?;

                let response_channel = format!("pos.ecr/{}", tid);
                println!("subscribed to nats response channel: {}", response_channel);
                let mut pos_subscriber = nats_client.subscribe(response_channel).await?;

                if let Some(pos_message) = pos_subscriber.next().await {
                    println!("received response from nats: {:?}", pos_message);
                    socket.write_all(pos_message.payload.as_ref()).await?;
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
async fn test_tcp_server_with_multiple_clients(
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    for i in 1..=10 {
        let nats_client = nats_client.clone();
        let tid = format!("tid{}", i);
        let response_message = format!("response_from_pos_{}", i);
        tokio::spawn(async move {
            let pos_channel = format!("ecr.pos/{}", tid);
            let mut pos_subscriber = nats_client.subscribe(pos_channel.clone()).await.unwrap();
            println!("pos client subscribed to {}", pos_channel);

            if let Some(pos_message) = pos_subscriber.next().await {
                println!(
                    "pos client received message on {}: {:?}",
                    pos_channel, pos_message
                );
                let response_channel = format!("pos.ecr/{}", tid);
                nats_client
                    .publish(response_channel, response_message.into())
                    .await
                    .unwrap();
            }
        });
    }

    let mut ecr_streams = vec![];
    for i in 1..=10 {
        let mut ecr_stream = TcpStream::connect("localhost:4000").await?;
        let message = format!("tid{}:message_from_ecr", i);
        ecr_stream.write_all(message.as_bytes()).await?;
        ecr_streams.push((i, ecr_stream));
    }

    for (i, mut ecr_stream) in ecr_streams {
        let mut buf = [0; 1024];
        let n = ecr_stream.read(&mut buf).await?;
        let response = std::str::from_utf8(&buf[..n])?;
        let expected_response = format!("response_from_pos_{}", i);
        assert_eq!(response, expected_response);
        println!("ecr client {} received correct response: {}", i, response);
    }

    shutdown_tx.send(()).await?;
    server.await??;

    Ok(())
}
