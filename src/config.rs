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
#[command(author, version, about)]
pub struct Config {
    #[command(flatten)]
    pub app: AppConfig,
}

impl Config {
    pub fn parse_config() -> Self {
        Self::parse()
    }
}
