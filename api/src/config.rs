use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct DatabaseConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct MessageBrokerConfig {
    pub host: String,
    pub port: u16,
    pub keep_alive: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub message_broker: MessageBrokerConfig,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let _ = dotenvy::dotenv();

        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .add_source(config::Environment::separator(
                config::Environment::with_prefix("API"),
                "_",
            ))
            .build()?;

        settings.try_deserialize()
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

// Type alias for shared configuration
pub type SharedConfig = Arc<Config>;
