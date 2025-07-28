use actix_web::{get, App, HttpServer, Responder, Result, web};
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

mod config;
use config::{Config, SharedConfig};

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
struct HelloResponse {
    message: String,
}

#[derive(ToSchema)]
#[derive(Debug, Serialize)]
struct ConfigResponse {
    server: config::ServerConfig,
    database: config::DatabaseConfig,
    message_broker: config::MessageBrokerConfig,
    services: config::ServicesConfig,
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
async fn api_test() -> Result<impl Responder> {
    let response = HelloResponse {
        message: "Hello from API endpoint!".to_string(),
    };
    Ok(actix_web::web::Json(response))
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
async fn get_config(config: web::Data<SharedConfig>) -> Result<impl Responder> {
    let response = ConfigResponse {
        server: config.server.clone(),
        database: config.database.clone(),
        message_broker: config.message_broker.clone(),
        services: config.services.clone(),
    };
    Ok(actix_web::web::Json(response))
}

#[derive(OpenApi)]
#[openapi(
    paths(api_test, get_config),
    components(schemas(HelloResponse, ConfigResponse)),
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
    // Load configuration
    let config = Config::load().expect("Failed to load configuration");
    let shared_config = std::sync::Arc::new(config);
    let server_address = shared_config.server_address();
    
    println!("Starting Rust API server...");
    println!("API endpoints:");
    println!("  - GET /api/test");
    println!("  - GET /config - View configuration");
    println!("  - GET /swagger-ui/ - Swagger UI documentation");
    println!("Server address: {}", server_address);
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_config.clone()))
            .service(api_test)
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
