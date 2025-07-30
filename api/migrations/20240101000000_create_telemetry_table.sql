-- Create telemetry table
CREATE TABLE IF NOT EXISTS telemetry (
    id TEXT PRIMARY KEY NOT NULL,
    time DATETIME NOT NULL,
    temperature REAL NOT NULL,
    voltage REAL NOT NULL,
    current REAL NOT NULL,
    battery_level INTEGER NOT NULL,
    created_at DATETIME NOT NULL
);

-- Create index on time for better query performance
CREATE INDEX IF NOT EXISTS idx_telemetry_time ON telemetry(time); 