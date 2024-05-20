mod server;

use std::env;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    set_up_logging()?;

    let port = env::var("PORT").unwrap_or_else(|_| "8080".into());

    let listener = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    info!("listening on {}", listener.local_addr().unwrap());

    server::run(listener, signal::ctrl_c()).await;

    Ok(())
}

fn set_up_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt::try_init()
}
