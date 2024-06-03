use crate::{Config, Connection, Nats, Shutdown};

use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct Handler {
    pub(crate) config: Arc<Config>,
    pub(crate) nats: Arc<Nats>,
    pub(crate) connection: Connection,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.shutdown.is_shutdown() {
            // debug!("{:?}", self.connection);

            let maybe_data = tokio::select! {
                res = self.connection.read_data() => res?,
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            };

            let data = match maybe_data {
                Some(data) => data,
                None => return Ok(()),
            };

            debug!("{:?}", data);

            self.nats
                .publish(self.config.nats.topic.transaction.to_string(), data.into())
                .await?;

            // self.nats.flush().await?;

            let mut subscription = self
                .nats
                .subscribe(self.config.nats.topic.result.to_string())
                .await?;

            tokio::select! {
                Some(message) = subscription.next() => {
                    debug!("Received message {message:?}");
                    self.connection.write_data(&message.payload).await?;
                }
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            }

            drop(subscription);
        }
        Ok(())
    }
}
