use crate::config::SharedConfig;
use crate::models::responses::ConfigResponse;
use actix_web::{get, web, Responder, Result};

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
    };
    Ok(actix_web::web::Json(response))
}
