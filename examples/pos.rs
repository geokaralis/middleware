use async_nats::jetstream::{self, stream};
use futures::StreamExt;

/// RUST_LOG=debug cargo run
///
/// cargo run --example pos
///
/// cargo run --example ecr

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            filter_subject: "events.ecr.>".to_string(),
            ..Default::default()
        })
        .await?;

    let mut messages = consumer.messages().await?;

    while let Some(message) = messages.next().await {
        let message = message?;
        println!("Received message {:?}", message);

        jetstream
            .publish(
                "events.pos.transaction",
                "ACQ103TID88824756\0\x0ePOS0110X/hellopos".into(),
            )
            .await?
            .await?;
        message.ack().await?;
    }

    Ok(())
}
