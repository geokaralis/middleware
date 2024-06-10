use async_nats::jetstream::{self, stream};
use futures::StreamExt;
use tracing::info;

/// RUST_LOG=debug cargo run
///
/// cargo run --example pos
///
/// cargo run --example ecr

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::try_init()?;

    let nats = async_nats::connect("nats://127.0.0.1:4222").await?;

    let jetstream = jetstream::new(nats);

    let stream = jetstream
        .create_stream(jetstream::stream::Config {
            name: "EVENTS".to_string(),
            retention: stream::RetentionPolicy::WorkQueue,
            subjects: vec!["events.>".to_string()],
            ..Default::default()
        })
        .await?;

    let consumer = stream
        .create_consumer(jetstream::consumer::pull::Config {
            durable_name: Some("pos-processor".to_string()),
            filter_subject: "events.ecr.88824756".to_string(),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        info!("received message {:?}", message);

        jetstream
            .publish(
                "events.pos.88824756",
                "ACQ103TID88824756\0\x0ePOS0110X/hellopos".into(),
            )
            .await?
            .await?;
        message.ack().await?;
    }

    Ok(())
}
