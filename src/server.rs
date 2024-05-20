use futures::Future;
use std::str;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
};
use tracing::{error, info};

#[derive(Debug)]
struct Listener {
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        listener,
        notify_shutdown,
        shutdown_complete_tx,
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down")
        }
    }

    let Listener {
        shutdown_complete_tx,
        notify_shutdown,
        ..
    } = server;

    drop(notify_shutdown);
    drop(shutdown_complete_tx);

    let _ = shutdown_complete_rx.recv().await;
}

impl Listener {
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("accepting inbound connections");

        loop {
            let mut socket = self.accept().await?;

            tokio::spawn(async move {
                let mut buf = [0; 4 * 1024];

                let n = socket.read(&mut buf).await.unwrap();

                socket.write_all(&buf[0..n]).await.unwrap();

                let data = str::from_utf8(&buf[0..n]).unwrap();
                info!(data);

                socket.shutdown().await.unwrap();
            });
        }
    }

    async fn accept(&mut self) -> Result<TcpStream, Box<dyn std::error::Error + Send + Sync>> {
        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    return Err(err.into());
                }
            }
        }
    }
}
