use clap::Parser;
use std::net::Ipv4Addr;

#[derive(Parser, Debug)]
pub struct AppConfig {
    #[arg(name = "hostname", long, env("APP_HOST"), default_value = "0.0.0.0")]
    pub host: Ipv4Addr,
    #[arg(short, long, env("APP_PORT"), default_value_t = 8080)]
    pub port: u16,
}

#[derive(Parser, Debug)]
pub struct NatsConfig {
    #[arg(
        name = "nats_url",
        long,
        env("NATS_URL"),
        default_value = "nats://127.0.0.1:4222"
    )]
    pub url: String,

    #[command(flatten)]
    pub topic: Topic,
}

#[derive(Parser, Debug)]
pub struct Topic {
    #[arg(
        name = "topic_transaction",
        long,
        env("NATS_TRANSACTION_TOPIC"),
        default_value = "transaction"
    )]
    pub transaction: String,

    #[arg(
        name = "topic_result",
        long,
        env("NATS_RESULT_TOPIC"),
        default_value = "transaction.result"
    )]
    pub result: String,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Config {
    #[command(flatten)]
    pub app: AppConfig,

    #[command(flatten)]
    pub nats: NatsConfig,
}

impl Config {
    pub fn parse_config() -> Self {
        Self::parse()
    }
}
