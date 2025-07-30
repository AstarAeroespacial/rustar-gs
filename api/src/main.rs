use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod routes;
mod models;

use config::{Config, ServerConfig, DatabaseConfig, MessageBrokerConfig, ServicesConfig};
use routes::{telemetry::get_telemetry, config::get_config};
use models::{requests::TelemetryRequest, responses::*};

#[derive(OpenApi)]
#[openapi(
    paths(routes::telemetry::get_telemetry, routes::config::get_config),
    components(schemas(
        TelemetryResponse,
        ConfigResponse,
        TelemetryRequest,
        ServerConfig,
        DatabaseConfig,
        MessageBrokerConfig,
        ServicesConfig
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
    
    println!("Starting Rust API server...");
    println!("API endpoints:");
    println!("  - GET /api/telemetry");
    println!("  - GET /config - View configuration");
    println!("  - GET /swagger-ui/ - Swagger UI documentation");
    println!("Server address: {}", server_address);
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_config.clone()))
            .service(get_telemetry)
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
