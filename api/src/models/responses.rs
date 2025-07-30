use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct TelemetryResponse {
    pub start_time: i64,
    pub end_time: i64,
    pub page_size: i32,
    pub page_number: i32
}

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub server: crate::config::ServerConfig,
    pub database: crate::config::DatabaseConfig,
    pub message_broker: crate::config::MessageBrokerConfig,
    pub services: crate::config::ServicesConfig,
}