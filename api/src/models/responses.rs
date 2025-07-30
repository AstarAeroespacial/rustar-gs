use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct TelemetryResponse {
    pub time: String, // ISO timestamp
    pub temperature: f64,
    pub voltage: f64,
    pub current: f64,
    pub battery_level: i32, // percentage
}

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub server: crate::config::ServerConfig,
    pub database: crate::config::DatabaseConfig,
    pub message_broker: crate::config::MessageBrokerConfig,
}