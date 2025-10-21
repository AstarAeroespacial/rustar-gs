use axum::{Json, response::IntoResponse};
use serde_json::json;
use utoipa::OpenApi;

use crate::job::{Job, TleData};

/// # API Documentation
///
/// `ApiDoc` generates the OpenAPI specification for the Ground Station API,
/// exposing endpoints for interacting with a ground station running instance.
#[derive(OpenApi)]
#[openapi(
    paths(
        add_job,
        root
    ),
    components(
        schemas(Job, TleData)
    ),
    tags(
        (name = "Ground Station API", description = "API for interacting with a running ground station instance")
    )
)]
pub struct ApiDoc;

#[utoipa::path(
    post,
    path = "/jobs",
    tag = "Jobs",
    request_body = Job,
    responses(
    )
)]
pub async fn add_job(
    axum::extract::State(job_tx): axum::extract::State<tokio::sync::mpsc::UnboundedSender<Job>>,
    Json(job): Json<Job>,
) -> impl IntoResponse {
    println!("[API] Received job request: {:#?}", &job);

    // Send job through channel
    if let Err(_) = job_tx.send(job) {
        eprintln!("Failed to send job to scheduler");

        return Json(json!({"status": "error", "message": "Failed to add job to scheduler"}));
    }

    Json(json!({"status": "ok", "message": "Job sent to scheduler successfully"}))
}

#[utoipa::path(get, path = "/", tag = "Ground Station", responses())]
pub async fn root() -> impl IntoResponse {
    Json(json!({ "status": "ok", "message": "Ground Station API is running 🚀" }))
}
