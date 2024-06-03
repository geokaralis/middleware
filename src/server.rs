use std::sync::Arc;

use crate::{Config, Connection, Handler, Nats, Shutdown};

use futures::Future;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
};
use tracing::{error, info};

#[derive(Debug)]
struct Listener {
    config: Arc<Config>,
    nats: Arc<Nats>,
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

pub async fn run(nats: Nats, listener: TcpListener, shutdown: impl Future, config: Config) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        config: Arc::new(config),
        nats: Arc::new(nats),
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
            let socket = self.accept().await?;

            let mut handler = Handler {
                config: self.config.clone(),
                nats: self.nats.clone(),
                connection: Connection::new(socket),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
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
