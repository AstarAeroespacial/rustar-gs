-- Create telemetry table
CREATE TABLE IF NOT EXISTS telemetry (
    id TEXT PRIMARY KEY NOT NULL,
    timestamp INTEGER NOT NULL,
    temperature REAL NOT NULL,
    voltage REAL NOT NULL,
    current REAL NOT NULL,
    battery_level INTEGER NOT NULL
);

-- Create index on time for better query performance
CREATE INDEX IF NOT EXISTS idx_telemetry_time ON telemetry(timestamp); 