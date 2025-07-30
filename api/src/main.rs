use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod routes;
mod models;
mod repository;
mod services;
mod database;

use config::{Config, ServerConfig, DatabaseConfig, MessageBrokerConfig};
use routes::{telemetry::{get_latest_telemetry, get_historic_telemetry}, config::get_config};
use models::{requests::{HistoricTelemetryRequest, LatestTelemetryRequest}, responses::*};
use repository::{telemetry::SqliteTelemetryRepository};
use services::telemetry_service::TelemetryService;
use database::create_pool;
    
#[derive(OpenApi)]
#[openapi(
    paths(routes::telemetry::get_latest_telemetry, routes::telemetry::get_historic_telemetry, routes::config::get_config),
    components(schemas(
        TelemetryResponse,
        ConfigResponse,
        HistoricTelemetryRequest,
        LatestTelemetryRequest,
        ServerConfig,
        DatabaseConfig,
        MessageBrokerConfig,
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
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");
    let shared_config = std::sync::Arc::new(config);
    let server_address = shared_config.server_address();
    
    // Create database pool
    let pool = create_pool(&shared_config.database.url)
        .await
        .expect("Failed to create database pool");
    
    // Create repository and service
    let repository = SqliteTelemetryRepository::new(pool);
    let service = TelemetryService::new(repository);
    let shared_service = std::sync::Arc::new(service);
    
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
            .app_data(web::Data::new(shared_service.clone()))
            .service(get_latest_telemetry)
            .service(get_historic_telemetry)
            .service(get_config)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
    })
    .bind(server_address)?
    .run()
    .await
}
