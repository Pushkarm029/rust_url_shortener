use std::env;

use crate::storage::StorageType;

#[derive(Debug, Clone)]
pub struct Config {
    pub storage_type: StorageType,
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub short_url_length: usize,
    pub enable_metrics: bool,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            storage_type: StorageType::Sqlite, // TODO: Make this configurable
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:url_shortener.db".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            short_url_length: env::var("SHORT_URL_LENGTH")
                .unwrap_or_else(|_| "6".to_string())
                .parse()
                .unwrap_or(6),
            enable_metrics: env::var("ENABLE_METRICS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        }
    }
}
