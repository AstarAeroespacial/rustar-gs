use crate::models::commands::TestMessage;
use crate::services::message_service::MessageService;
use actix_web::{post, web, Responder, Result};
use log::error;
use std::sync::Arc;

#[utoipa::path(
    post,
    path = "/api/control/command",
    request_body = TestMessage,
    responses(
        (status = 200, description = "Success", body = Vec<TelemetryResponse>),
        (status = 400, description = "Bad Request", body = String),
        (status = 500, description = "Internal Server Error", body = String)
    ),
    tag = "API"
)]
#[post("/api/control/command")]
pub async fn send_command(
    req_body: web::Json<TestMessage>,
    service: web::Data<Arc<MessageService>>,
) -> Result<impl Responder> {
    println!("req_body: {:?}", req_body);
    let command = req_body;
    let command_str = serde_json::to_string(&command);
    match command_str {
        Ok(command_str) => match service.send_message("test-topic", &command_str).await {
            Ok(_) => Ok(actix_web::HttpResponse::Ok().body("Message sent successfully")),
            Err(e) => {
                error!("Error sending message: {}", e);
                Err(actix_web::error::ErrorInternalServerError(
                    "Failed to send message",
                ))
            }
        },
        Err(e) => {
            error!("Error serializing command: {}", e);
            Err(actix_web::error::ErrorBadRequest(
                "Failed to serialize command",
            ))
        }
    }
}
