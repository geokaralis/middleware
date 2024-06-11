use crate::{Config, Connection, Handler, Shutdown};

use async_nats::jetstream::{
    self,
    consumer::pull::Config as ConsumerConfig,
    consumer::Consumer,
    stream::{self},
    Context as JetStreamContext,
};
use futures::Future;
use std::{sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
};
use tracing::{error, info};

#[derive(Debug)]
struct Listener {
    config: Arc<Config>,
    jetstream: JetStreamContext,
    consumer: Consumer<ConsumerConfig>,
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

pub async fn run(
    jetstream: JetStreamContext,
    listener: TcpListener,
    shutdown: impl Future,
    config: Config,
) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let stream = jetstream
        .create_stream(jetstream::stream::Config {
            name: config.nats.stream.clone(),
            retention: stream::RetentionPolicy::WorkQueue,
            subjects: vec![config.nats.subjects.clone()],
            max_age: Duration::from_secs(60),
            ..Default::default()
        })
        .await
        .unwrap();

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some(config.nats.consumer.durable_name.clone()),
            filter_subject: config.nats.consumer.filter_subject.clone(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut server = Listener {
        config: Arc::new(config),
        jetstream,
        consumer,
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
                jetstream: self.jetstream.clone(),
                consumer: self.consumer.clone(),
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
