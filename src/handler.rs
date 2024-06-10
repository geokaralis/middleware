use crate::{utils::extract_tid, Config, Connection, Shutdown};

use async_nats::jetstream::{
    consumer::{pull::Config as ConsumerConfig, Consumer},
    Context as JetStreamContext,
};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct Handler {
    pub(crate) config: Arc<Config>,
    pub(crate) jetstream: JetStreamContext,
    pub(crate) consumer: Consumer<ConsumerConfig>,
    pub(crate) connection: Connection,
    pub(crate) shutdown: Shutdown,
    pub(crate) _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    pub(crate) async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while !self.shutdown.is_shutdown() {
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

            let tid = match extract_tid(&data) {
                Some(data) => data,
                None => return Ok(()),
            };

            self.jetstream
                .publish(
                    format!("{}.{}", self.config.nats.topic.ecr, tid),
                    data.clone().into(),
                )
                .await?
                .await?;

            let mut messages = self.consumer.messages().await?;

            tokio::select! {
                Some(message) = messages.next() => {
                    let message = message?;
                    if message.subject == format!("{}.{}", self.config.nats.topic.pos, tid).into() {
                        debug!("received message {:?}", message);
                        self.connection.write_data(&message.payload).await?;
                        message.ack().await?;
                    }
                }
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            }
        }

        Ok(())
    }
}
