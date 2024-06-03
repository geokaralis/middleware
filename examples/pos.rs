use middleware::Config;

use futures::StreamExt;

/// RUST_LOG=debug cargo run
///
/// cargo run --example pos
///
/// cargo run --example ecr

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::parse_config();
    let nats = async_nats::connect("nats://127.0.0.1:4222").await?;

    let mut subscription = nats.subscribe(config.nats.topic.transaction).await?;

    if let Some(message) = subscription.next().await {
        println!("Received message {message:?}");

        nats.publish(config.nats.topic.result, "world".into())
            .await?;
    }

    Ok(())
}
