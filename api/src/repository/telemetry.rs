use sqlx::{Any, Pool};
use chrono::Utc;
use crate::models::telemetry::TelemetryRecord;

pub struct TelemetryRepository {
    pool: Pool<Any>,
}

impl TelemetryRepository {
    pub fn new(pool: Pool<Any>) -> Self {
        Self { pool }
    }

    pub async fn get_latest(&self, limit: i32) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let records = sqlx::query_as(
            r#"
            SELECT id, timestamp, temperature, voltage, current, battery_level
            FROM telemetry
            ORDER BY timestamp DESC
            LIMIT ?
            "#
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_historic(&self, start_time: Option<i64>, end_time: Option<i64>) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let start_ts = start_time.unwrap_or(0);
        let end_ts = end_time.unwrap_or(Utc::now().timestamp());

        let records = sqlx::query_as(
                r#"
                SELECT id, timestamp, temperature, voltage, current, battery_level
                FROM telemetry
                WHERE timestamp >= ? AND timestamp <= ?
                ORDER BY timestamp DESC
                "#
        )
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    // async fn save(&self, telemetry: TelemetryRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //     sqlx::query(
    //         r#"
    //         INSERT INTO telemetry (id, time, temperature, voltage, current, battery_level, created_at)
    //         VALUES (?, ?, ?, ?, ?, ?, ?)
    //         "#
    //     )
    //     .bind(telemetry.id)
    //     .bind(telemetry.timestamp)
    //     .bind(telemetry.temperature)
    //     .bind(telemetry.voltage)
    //     .bind(telemetry.current)
    //     .bind(telemetry.battery_level)
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(())
    // }
} 