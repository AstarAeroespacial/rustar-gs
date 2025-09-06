-- Add migration script here
ALTER TABLE telemetry
    ALTER COLUMN timestamp TYPE BIGINT;
