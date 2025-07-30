use actix_web::{get, Responder, Result, web};

use crate::models::{responses::TelemetryResponse, requests::TelemetryRequest};

/// Telemetry endpoint
#[utoipa::path(
    get,
    path = "/api/telemetry",
    params(
        ("startTime" = Option<i64>, Query, description = "Start timestamp", example = 1640995200),
        ("endTime" = Option<i64>, Query, description = "End timestamp", example = 1640998800),
        ("pageSize" = Option<i32>, Query, description = "Number of items per page", example = 10),
        ("pageNumber" = Option<i32>, Query, description = "Page number", example = 1)
    ),
    responses(
        (status = 200, description = "Success", body = TelemetryResponse),
        (status = 400, description = "Bad Request", body = String)
    ),
    tag = "API"
)]
#[get("/api/telemetry")]
pub async fn get_telemetry(req: web::Query<TelemetryRequest>) -> Result<impl Responder> {
    let req = req.into_inner();
    Ok(actix_web::web::Json(TelemetryResponse {
        start_time: req.start_time.unwrap_or(0),
        end_time: req.end_time.unwrap_or(0),
        page_size: req.page_size.unwrap_or(10),
        page_number: req.page_number.unwrap_or(1)
    }))
} 