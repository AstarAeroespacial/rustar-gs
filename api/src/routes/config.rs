use actix_web::{get, Responder, Result, web};
use serde::Serialize;
use utoipa::ToSchema;
use crate::config::SharedConfig;

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    server: crate::config::ServerConfig,
    database: crate::config::DatabaseConfig,
    message_broker: crate::config::MessageBrokerConfig,
    services: crate::config::ServicesConfig,
}

/// Configuration endpoint
#[utoipa::path(
    get,
    path = "/config",
    responses(
        (status = 200, description = "Success", body = ConfigResponse)
    ),
    tag = "Config"
)]
#[get("/config")]
pub async fn get_config(config: web::Data<SharedConfig>) -> Result<impl Responder> {
    let response = ConfigResponse {
        server: config.server.clone(),
        database: config.database.clone(),
        message_broker: config.message_broker.clone(),
        services: config.services.clone(),
    };
    Ok(actix_web::web::Json(response))
} 