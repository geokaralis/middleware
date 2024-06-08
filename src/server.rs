use crate::{Connection, Handler, Shutdown};

use async_nats::jetstream::{
    self,
    consumer::pull::Config as ConsumerConfig,
    consumer::Consumer,
    stream::{self},
    Context as JetStreamContext,
};
use futures::Future;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{broadcast, mpsc},
};
use tracing::{debug, error, info};

#[derive(Debug)]
struct Listener {
    // config: Arc<Config>,
    jetstream: JetStreamContext,
    consumer: Consumer<ConsumerConfig>,
    // nats: Arc<Nats>,
    listener: TcpListener,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

pub async fn run(
    jetstream: JetStreamContext,
    listener: TcpListener,
    shutdown: impl Future,
    // config: Config,
) {
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let stream = jetstream
        .create_stream(jetstream::stream::Config {
            name: "EVENTS".to_string(),
            retention: stream::RetentionPolicy::WorkQueue,
            subjects: vec!["events.>".to_string()],
            ..Default::default()
        })
        .await
        .unwrap();
    debug!("Created the stream: {:?}", stream);

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("middleware-processor".to_string()),
            filter_subject: "events.pos.>".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();
    debug!("Created the consumer: {:?}", consumer);

    let mut server = Listener {
        // config: Arc::new(config),
        jetstream,
        consumer,
        // nats: Arc::new(nats),
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
                // config: self.config.clone(),
                jetstream: self.jetstream.clone(),
                consumer: self.consumer.clone(),
                // nats: self.nats.clone(),
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
