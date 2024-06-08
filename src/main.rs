use async_nats::jetstream;
use middleware::{server, Config};

use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_up_logging()?;

    let config = Config::parse_config();

    let nats = async_nats::connect(&config.nats.url).await?;
    let jetstream = jetstream::new(nats);

    let addr = format!("{}:{}", config.app.host, config.app.port);

    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", listener.local_addr().unwrap());

    server::run(jetstream, listener, signal::ctrl_c()).await;

    Ok(())
}

fn set_up_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::try_init()
}
