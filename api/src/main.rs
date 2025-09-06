use actix_web::{middleware::Logger, web, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod database;
mod messaging;
mod models;
mod repository;
mod routes;
mod services;

use config::{Config, DatabaseConfig, MessageBrokerConfig, ServerConfig};
use database::create_pool;
use messaging::{broker::MqttBroker, receiver::MqttReceiver};
use models::{
    commands::TestMessage,
    requests::{HistoricTelemetryRequest, LatestTelemetryRequest},
    responses::*,
};
use repository::telemetry::TelemetryRepository;
use routes::{
    config::get_config,
    control::send_command,
    telemetry::{get_historic_telemetry, get_latest_telemetry},
};
use services::{message_service::MessageService, telemetry_service::TelemetryService};
use tokio::signal;
use tokio::sync::oneshot;

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
    println!("Database url: {}", &shared_config.database.url);

    let pool = create_pool(&shared_config.database.url)
        .await
        .expect("Failed to create database pool");

    // Create repository and service
    let repository = TelemetryRepository::new(pool);
    let telemetry_service = std::sync::Arc::new(TelemetryService::new(repository));

    let keepalive = std::time::Duration::from_secs(shared_config.message_broker.keep_alive as u64);
    let (broker, eventloop) = MqttBroker::new(
        &shared_config.message_broker.host,
        shared_config.message_broker.port,
        keepalive,
    );
    let client = broker.client();
    let messaging_service = std::sync::Arc::new(MessageService::new(broker));

    // Start event loop in a separate thread
    let mut recv = MqttReceiver::from_client(client, eventloop, telemetry_service.clone());

    println!("============= THIS NEEDS TO BE UPDATED =============");
    println!("Starting Rust API server...");
    println!("API endpoints:");
    println!("  - GET /api/telemetry/latest");
    println!("  - GET /api/telemetry/history");
    println!("  - GET /config - View configuration");
    println!("  - GET /swagger-ui/ - Swagger UI documentation");
    println!("Server address: {}", server_address);
    println!("============= THIS NEEDS TO BE UPDATED =============");
    
    let server = HttpServer::new(move || {
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
    .bind(server_address)?;

    // Create shutdown channel for MQTT receiver
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Start HTTP server and obtain handle
    let server_handle = server.run();
    let handle = server_handle.handle();

    // Spawn MQTT receiver task, using tokio::task::spawn_blocking to avoid Send/Sync issues
    let recv_task = tokio::task::spawn_blocking(move || {
        // Since run is async, we need a runtime here
        let rt =
            tokio::runtime::Runtime::new().expect("Failed to create runtime for MQTT receiver");
        rt.block_on(recv.run(shutdown_rx));
    });

    // Wait for either ctrl+c or server error
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("SIGINT received: shutting down server and MQTT receiver...");
            let _ = shutdown_tx.send(());
            handle.stop(true).await;
        }
        res = server_handle => {
            if let Err(e) = res {
                eprintln!("HTTP server error: {:?}", e);
            }
            // Server finished; signal MQTT to stop as well
            let _ = shutdown_tx.send(());
        }
    }

    // Wait for MQTT task to finish
    let _ = recv_task.await;

    Ok(())
}
