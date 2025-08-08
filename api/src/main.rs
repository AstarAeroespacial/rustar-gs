use actix_web::{App, HttpServer, web, middleware::Logger};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use sqlx::any::install_default_drivers;

mod config;
mod routes;
mod models;
mod repository;
mod services;
mod database;
mod messaging;

use config::{Config, ServerConfig, DatabaseConfig, MessageBrokerConfig};
use routes::{telemetry::{get_latest_telemetry, get_historic_telemetry}, config::get_config, control::send_command};
use models::{requests::{HistoricTelemetryRequest, LatestTelemetryRequest}, responses::*, commands::TestMessage};
use repository::{telemetry::TelemetryRepository};
use services::{telemetry_service::TelemetryService, message_service::MessageService};
use database::create_pool;
use messaging::broker::MqttBroker;
    
#[derive(OpenApi)]
#[openapi(
    paths(routes::telemetry::get_latest_telemetry, routes::telemetry::get_historic_telemetry, routes::config::get_config, routes::control::send_command),
    components(schemas(
        TelemetryResponse,
        ConfigResponse,
        HistoricTelemetryRequest,
        LatestTelemetryRequest,
        ServerConfig,
        DatabaseConfig,
        MessageBrokerConfig,
        TestMessage
    )),
    tags(
        (name = "API", description = "Main API endpoints"),
        (name = "Config", description = "Configuration endpoints")
    ),
    info(
        title = "Rust API with Utoipa",
        version = "1.0.0",
        description = "A Rust API with OpenAPI documentation using Utoipa"
    )
)]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // std::env::set_var("RUST_LOG", "debug");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");
    let shared_config = std::sync::Arc::new(config);
    let server_address = shared_config.server_address();
    
    // Create database pool
    println!("Creating database pool...");

    install_default_drivers();
    
    let pool = create_pool(&shared_config.database.url)
        .await
        .expect("Failed to create database pool");
    
    // Create repository and service
    let repository = TelemetryRepository::new(pool);
    let telemetry_service = std::sync::Arc::new(TelemetryService::new(repository));

    let keepalive = std::time::Duration::from_secs(shared_config.message_broker.keep_alive as u64);
    let (broker, mut eventloop) = MqttBroker::new(&shared_config.message_broker.host, shared_config.message_broker.port, keepalive);
    let messaging_service = std::sync::Arc::new(MessageService::new(broker));
    

    // Start event loop in a separate thread
    let _eventloop_thread = tokio::spawn(async move {
        loop {
            let result = eventloop.poll().await;
            if let Err(e) = result {
                println!("Event loop error: {}", e);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    println!("Starting Rust API server...");
    println!("API endpoints:");
    println!("  - GET /api/telemetry/latest");
    println!("  - GET /api/telemetry/history");
    println!("  - GET /config - View configuration");
    println!("  - GET /swagger-ui/ - Swagger UI documentation");
    println!("Server address: {}", server_address);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_config.clone()))
            .app_data(web::Data::new(telemetry_service.clone()))
            .app_data(web::Data::new(messaging_service.clone()))
            .service(get_latest_telemetry)
            .service(get_historic_telemetry)
            .service(get_config)
            .service(send_command)
            .wrap(Logger::new("%r - %U | %s (%T)"))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(server_address)?
    .run()
    .await
}
