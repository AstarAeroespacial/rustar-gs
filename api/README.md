# Rust API Server

TODO: This file needs a rewrite

A Rust API server built with Actix-web that includes configuration management, database integration, and OpenAPI documentation.

## Architecture

The application follows a clean architecture pattern with the following layers:

- **Routes**: Handle HTTP requests and responses
- **Services**: Business logic layer for exchanging messages and accessing telemetry
- **Repository**: Data access layer with database abstraction
- **Messaging**: Communication layer for interacting with ground stations
- **Models**: Data structures and DTOs

## Technology Stack

- `actix_web` and `utoipa` for the HTTP server and API documentation respectively
- `rumqttc` for MQTT integration
- `sqlx` for postgres integration
- The app was developed with `psql` for the database and `mosquitto` for the MQTT broker

## Configuration

The server uses a `config.toml` file for configuration. The following sections are available:

### Server Configuration
- `host`: Server host address (default: 127.0.0.1)
- `port`: Server port (default: 8080)

### Database Configuration
- `url`: Database connection string
- `pool_size`: Connection pool size

### Message Broker Configuration
- `host`: Message broker address
- `port`: Port for the connection
- `keep_alive`: keepalive message interval

## Environment Variables

You can (and should) override configuration values using environment variables with the `API_` prefix:

```bash
export API_SERVER_HOST=0.0.0.0
export API_SERVER_PORT=3000
export API_DATABASE_URL=sqlite:./data/telemetry.db
```

It is recomended that you create a `.env` file with all your local development config. You can refer to the provided `.env.example`

## Setup for Local development

### MQTT Broker Setup

1. Install any MQTT broker of your choosing. For development, we used [mosquitto](https://www.mosquitto.org/download/).
2. Open another terminal and run the broker. With mosquitto, this is done with the command `mosquitto -p 1234`.
3. Make sure to set the host and port appropriately in the config 

### Database Setup

1. Install a [Postgres database](https://www.postgresql.org/download/) via the method of your choosing
2. Install [sqlx cli tool](https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md) by running `cargo install sqlx-cli`
3. Add the database path by running `export DATABASE_URL=postgresql://localhost/rustar-api?user=myuser&password=mypw` (or whichever path you have for your database). It's recommended to add this variable to a .env file (Note that this is different from API_DATABASE_URL, both should be set)
4. Initialize the database by running `sqlx database create`
5. Run migrations with `sqlx migrate run`
6. You can generate test data with `cargo run --bin seed_data`

### Running the Server

1. Copy the `.env.example` file to `.env` and make sure all the variables are set
2. Run the server:
   ```bash
   cargo run --bin api
   ```

## API Endpoints

- `GET /api/telemetry/latest?amount=10` - Get latest telemetry data
- `GET /api/telemetry/history?startTime=1640995200&endTime=1640998800` - Get historic telemetry data
- `GET /config` - View current configuration
- `GET /swagger-ui/` - OpenAPI documentation

## Development

The server is structured with:
- `src/main.rs` - Main application entry point
- `src/config.rs` - Configuration management
- `src/models/` - Data models and DTOs
- `src/repository/` - Database access layer
- `src/services/` - Business logic layer
- `src/routes/` - HTTP route handlers
- `src/database/` - Database connection management
- `src/messaging/` - MQTT integration
- `migrations/` - Database schema migrations
