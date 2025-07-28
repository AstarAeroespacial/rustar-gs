# Rust API Server

A Rust API server built with Actix-web that includes configuration management and OpenAPI documentation.

## Features

- Configuration management using TOML files
- OpenAPI/Swagger documentation with Utoipa
- Environment variable support for configuration
- Shared configuration across the application

## Configuration

The server uses a `config.toml` file for configuration. The following sections are available:

### Server Configuration
- `host`: Server host address (default: 127.0.0.1)
- `port`: Server port (default: 8080)

### Database Configuration
- `url`: Database connection string
- `pool_size`: Connection pool size

### Message Broker Configuration
- `url`: Message broker connection string
- `queue_name`: Queue name for receiving messages
- `exchange_name`: Exchange name for publishing messages

### Services Configuration
- `external_api_url`: External API service URL
- `notification_service_url`: Notification service URL

## Environment Variables

You can override configuration values using environment variables with the `API_` prefix:

```bash
export API_SERVER_HOST=0.0.0.0
export API_SERVER_PORT=3000
export API_DATABASE_URL=postgresql://user:pass@localhost:5432/mydb
```

## API Endpoints

- `GET /api/test` - Test endpoint
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
- `config.toml` - Configuration file

## Next Steps

1. Add database connection and models
2. Implement message broker integration
3. Add authentication and authorization
4. Implement the actual business logic endpoints 