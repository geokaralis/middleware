// use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::{broadcast, mpsc};
// use tokio::sync::Semaphore;
use std::io;
use tracing::{error, info};

async fn handle_connection(
    socket: &mut TcpStream,
    // mut permit: &mut tokio::sync::OwnedSemaphorePermit,
) {
    let mut buf = [0; 1024];

    loop {
        let n = match socket.read(&mut buf).await {
            // socket closed
            Ok(n) if n == 0 => return,
            Ok(n) => n,
            Err(e) => {
                error!("failed to read from socket; err = {:?}", e);
                return;
            }
        };

        // Write the data back
        if let Err(e) = socket.write_all(&buf[0..n]).await {
            error!("failed to write to socket; err = {:?}", e);
            return;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let (notify_shutdown, _) = broadcast::channel::<()>(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel::<()>(1);

    // let semaphore = Arc::new(Semaphore::new(100));
    let listener = TcpListener::bind("0.0.0.0:4000").await?;
    info!("server listening on 0.0.0.0:4000");

    tokio::select! {
        _ = async {
            loop {
                // let mut permit = semaphore.clone().acquire_owned().await.unwrap();
                let (mut socket, _) = listener.accept().await?;

                tokio::spawn(async move {
                    handle_connection(&mut socket).await;
                    drop(socket);
                    // drop(permit);
                });
            }
            Ok::<_, io::Error>(())
        } => {}
        _ = signal::ctrl_c() => {
            // The shutdown signal has been received.
            info!("shutting down");
        }
    }

    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_complete_tx);

    let _: Option<_> = shutdown_complete_rx.recv().await;

    Ok(())
}
