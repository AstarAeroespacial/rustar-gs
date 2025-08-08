# Rust API Server

TODO: This file needs a rewrite

A Rust API server built with Actix-web that includes configuration management, database integration, and OpenAPI documentation.

## Architecture

The application follows a clean architecture pattern with the following layers:

- **Controllers (Routes)**: Handle HTTP requests and responses
- **Services**: Business logic layer
- **Repository**: Data access layer with database abstraction
- **Models**: Data structures and DTOs

## Features

- Configuration management using TOML files
- OpenAPI/Swagger documentation with Utoipa
- Environment variable support for configuration
- Database abstraction layer (currently SQLite, easily swappable)
- Service-controller architecture for clean separation of concerns
- Database migrations with SQLx

## Configuration

The server uses a `config.toml` file for configuration. The following sections are available:

### Server Configuration
- `host`: Server host address (default: 127.0.0.1)
- `port`: Server port (default: 8080)

### Database Configuration
- `url`: Database connection string (default: sqlite:./.data/telemetry.db for local development)
- `pool_size`: Connection pool size

### Message Broker Configuration
- `url`: Message broker connection string
- `queue_name`: Queue name for receiving messages
- `exchange_name`: Exchange name for publishing messages

Currently unused

## Environment Variables

You can override configuration values using environment variables with the `API_` prefix:

```bash
export API_SERVER_HOST=0.0.0.0
export API_SERVER_PORT=3000
export API_DATABASE_URL=sqlite:./data/telemetry.db
```

## Database Setup

The application uses SQLite by default, but the repository pattern makes it easy to switch to other SQL databases.

### Initial Database Setup

1. Install [sqlx cli tool](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) by running `cargo install sqlx-cli`
2. Add the database path by running `export DATABASE_URL=sqlite:./.data/telemetry.db` (or whichever path you prefer)
3. Initialize the database by running `sqlx database create`
4. Run migrations with `sqlx migrate run`
5. You can generate test data with `cargo run --bin seed_data`

## API Endpoints

- `GET /api/telemetry/latest?amount=10` - Get latest telemetry data
- `GET /api/telemetry/history?startTime=1640995200&endTime=1640998800` - Get historic telemetry data
- `GET /config` - View current configuration
- `GET /swagger-ui/` - OpenAPI documentation

## Running the Server

1. Update the `config.toml` file with your actual configuration values
2. Run the server:
   ```bash
   cargo run
   ```

## Development

The server is structured with:
- `src/main.rs` - Main application entry point
- `src/config.rs` - Configuration management
- `src/models/` - Data models and DTOs
- `src/repository/` - Database access layer
- `src/services/` - Business logic layer
- `src/routes/` - HTTP route handlers
- `src/database/` - Database connection management
- `migrations/` - Database schema migrations

## Database Abstraction

The repository pattern allows easy database switching:

1. Create a new repository implementation (e.g., `PostgresTelemetryRepository`)
2. Implement the `TelemetryRepository` trait
3. Update the dependency injection in `main.rs`

## Next Steps

1. Add message broker integration
2. Implement authentication and authorization
3. Add more telemetry endpoints (create, update, delete)
4. Add data validation and error handling
5. Implement caching layer 