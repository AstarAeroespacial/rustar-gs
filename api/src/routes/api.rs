use actix_web::{get, Responder, Result};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
pub struct HelloResponse {
    message: String,
}

/// API test endpoint
#[utoipa::path(
    get,
    path = "/api/test",
    responses(
        (status = 200, description = "Success", body = HelloResponse)
    ),
    tag = "API"
)]
#[get("/api/test")]
pub async fn api_test() -> Result<impl Responder> {
    let response = HelloResponse {
        message: "Hello from API endpoint!".to_string(),
    };
    Ok(actix_web::web::Json(response))
} 