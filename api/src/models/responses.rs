use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema, Debug, Serialize)]
pub struct TelemetryResponse {
    pub timestamp: i64, // ISO timestamp
    pub temperature: f32,
    pub voltage: f32,
    pub current: f32,
    pub battery_level: i32, // percentage
}

#[derive(ToSchema, Debug, Serialize)]
pub struct ConfigResponse {
    pub server: crate::config::ServerConfig,
    pub database: crate::config::DatabaseConfig,
    pub message_broker: crate::config::MessageBrokerConfig,
}
