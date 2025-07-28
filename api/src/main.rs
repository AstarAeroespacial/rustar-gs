use actix_web::{App, HttpServer, web};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod routes;

use config::Config;
use routes::{api_test, get_config, HelloResponse, ConfigResponse};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    #[derive(OpenApi)]
    #[openapi(
        paths(routes::api_test, routes::get_config),
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
