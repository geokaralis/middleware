use middleware::{server, Config, Nats};

use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_up_logging()?;

    let config = Config::parse_config();

    let nats = Nats::connect(&config.nats.url).await?;

    let addr = format!("{}:{}", config.app.host, config.app.port);

    let listener = TcpListener::bind(&addr).await?;
    info!("listening on {}", listener.local_addr().unwrap());

    server::run(nats, listener, signal::ctrl_c(), config).await;

    Ok(())
}

fn set_up_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::try_init()
}
