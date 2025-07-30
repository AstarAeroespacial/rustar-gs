use async_trait::async_trait;
use sqlx::{Pool, Sqlite};
use chrono::{Utc, TimeZone};
use crate::models::telemetry::TelemetryRecord;
use super::TelemetryRepository;

pub struct SqliteTelemetryRepository {
    pool: Pool<Sqlite>,
}

impl SqliteTelemetryRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TelemetryRepository for SqliteTelemetryRepository {
    async fn get_latest(&self, limit: i32) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let records = sqlx::query_as(
            r#"
            SELECT id, time, temperature, voltage, current, battery_level, created_at
            FROM telemetry
            ORDER BY time DESC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn get_historic(&self, start_time: Option<i64>, end_time: Option<i64>) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let start_ts = start_time.unwrap_or(Utc::now().timestamp());
        let end_ts = end_time.unwrap_or(Utc::now().timestamp());

        let start_dt = Utc.timestamp_opt(start_ts, 0).unwrap();
        let end_dt = Utc.timestamp_opt(end_ts, 0).unwrap();

        // For simplicity, we'll use a different approach with sqlx
        let records = sqlx::query_as(
                r#"
                SELECT id, time, temperature, voltage, current, battery_level, created_at
                FROM telemetry
                WHERE time >= ? AND time <= ?
                ORDER BY time DESC
                "#
        )
        .bind(start_dt)
        .bind(end_dt)
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
    //     .bind(telemetry.time)
    //     .bind(telemetry.temperature)
    //     .bind(telemetry.voltage)
    //     .bind(telemetry.current)
    //     .bind(telemetry.battery_level)
    //     .bind(telemetry.created_at)
    //     .execute(&self.pool)
    //     .await?;

    //     Ok(())
    // }
} 