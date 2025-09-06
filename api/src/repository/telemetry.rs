use crate::models::telemetry::TelemetryRecord;
use chrono::Utc;
use sqlx::{Pool, Postgres};

pub struct TelemetryRepository {
    pool: Pool<Postgres>,
}

impl TelemetryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    pub async fn get_latest(
        &self,
        sat_name: String,
        limit: i32,
    ) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let records = sqlx::query_as!(
            TelemetryRecord,
            r#"
            SELECT id, timestamp, temperature, voltage, current, battery_level
            FROM telemetry
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
            limit as i64
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_historic(
        &self,
        sat_name: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let start_ts = start_time.unwrap_or(0);
        let end_ts = end_time.unwrap_or(Utc::now().timestamp());

        let records = sqlx::query_as!(
            TelemetryRecord,
            r#"
            SELECT id, timestamp, temperature, voltage, current, battery_level
            FROM telemetry
            WHERE timestamp >= $1 AND timestamp <= $2
            ORDER BY timestamp DESC
            "#,
            start_ts,
            end_ts
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn save(
        &self,
        telemetry: TelemetryRecord,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        sqlx::query!(
            r#"
            INSERT INTO telemetry (id, timestamp, temperature, voltage, current, battery_level)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            telemetry.id,
            telemetry.timestamp,
            telemetry.temperature,
            telemetry.voltage,
            telemetry.current,
            telemetry.battery_level
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
