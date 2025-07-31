use actix_web::{get, Responder, Result, web};
use crate::models::{requests::{HistoricTelemetryRequest, LatestTelemetryRequest}};
use crate::services::telemetry_service::TelemetryService;
use log::error;
use std::sync::Arc;

/// Latest telemetry endpoint
#[utoipa::path(
    get,
    path = "/api/telemetry/latest",
    params(
        ("amount" = Option<i32>, Query, description = "Number of items to return", example = 10),
    ),
    responses(
        (status = 200, description = "Success", body = Vec<TelemetryResponse>),
        (status = 400, description = "Bad Request", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    ),
    tag = "API"
)]
#[get("/api/telemetry/latest")]
pub async fn get_latest_telemetry(
    req: web::Query<LatestTelemetryRequest>,
    service: web::Data<Arc<TelemetryService>>
) -> Result<impl Responder> {
    let req = req.into_inner();
    let amount = req.amount.unwrap_or(10);
    
    match service.get_latest_telemetry(amount).await {
        Ok(telemetry) => Ok(actix_web::web::Json(telemetry)),
        Err(e) => {
            error!("Error fetching latest telemetry: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to fetch telemetry data"))
        }
    }
}

/// Historic telemetry endpoint
#[utoipa::path(
    get,
    path = "/api/telemetry/history",
    params(
        ("startTime" = Option<i64>, Query, description = "Start timestamp", example = 1640995200),
        ("endTime" = Option<i64>, Query, description = "End timestamp", example = 1640998800),
    ),
    responses(
        (status = 200, description = "Success", body = Vec<TelemetryResponse>),
        (status = 400, description = "Bad Request", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    ),
    tag = "API"
)]
#[get("/api/telemetry/history")]
pub async fn get_historic_telemetry(
    req: web::Query<HistoricTelemetryRequest>,
    service: web::Data<Arc<TelemetryService>>
) -> Result<impl Responder> {
    let req = req.into_inner();
    
    match service.get_historic_telemetry(req.start_time, req.end_time).await {
        Ok(telemetry) => Ok(actix_web::web::Json(telemetry)),
        Err(e) => {
            error!("Error fetching historic telemetry: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to fetch telemetry data"))
        }
    }
} 