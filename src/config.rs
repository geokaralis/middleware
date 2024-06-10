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

    #[arg(
        name = "nats_stream",
        long,
        env("NATS_STREAM"),
        default_value = "EVENTS"
    )]
    pub stream: String,

    #[arg(
        name = "nats_subjects",
        long,
        env("NATS_SUBJECTS"),
        default_value = "events.>"
    )]
    pub subjects: String,

    #[command(flatten)]
    pub topic: Topic,

    #[command(flatten)]
    pub consumer: Consumer,
}

#[derive(Parser, Debug)]
pub struct Consumer {
    #[arg(
        name = "consumer_durable_name",
        long,
        env("NATS_CONSUMER_DURABLE_NAME"),
        default_value = "middleware-processor"
    )]
    pub durable_name: String,

    #[arg(
        name = "consumer_filter_subject",
        long,
        env("NATS_CONSUMER_FILTER_SUBJECT"),
        default_value = "events.pos.>"
    )]
    pub filter_subject: String,
}

#[derive(Parser, Debug)]
pub struct Topic {
    #[arg(
        name = "topic_ecr",
        long,
        env("NATS_ECR_TOPIC"),
        default_value = "events.ecr"
    )]
    pub ecr: String,

    #[arg(
        name = "topic_pos",
        long,
        env("NATS_POS_TOPIC"),
        default_value = "events.pos"
    )]
    pub pos: String,
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
