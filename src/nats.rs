use async_nats::{
    client::FlushError, Client as NatsClient, PublishError, SubscribeError, Subscriber,
};
use bytes::Bytes;
use std::error::Error;

#[derive(Debug)]
pub struct Nats {
    client: NatsClient,
}

impl Nats {
    pub async fn connect(url: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let client = async_nats::connect(url).await?;
        Ok(Nats { client })
    }

    pub async fn publish(&self, subject: String, payload: Bytes) -> Result<(), PublishError> {
        self.client.publish(subject, payload).await?;
        Ok(())
    }

    pub async fn subscribe(&self, subject: String) -> Result<Subscriber, SubscribeError> {
        let subscriber = self.client.subscribe(subject).await?;
        Ok(subscriber)
    }

    pub async fn flush(&self) -> Result<(), FlushError> {
        self.client.flush().await?;
        Ok(())
    }
}
