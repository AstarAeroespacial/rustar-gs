pub mod telemetry;

use async_trait::async_trait;
use crate::models::telemetry::TelemetryRecord;

#[async_trait]
pub trait TelemetryRepository {
    async fn get_latest(&self, limit: i32) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>>;
    async fn get_historic(&self, start_time: Option<i64>, end_time: Option<i64>) -> Result<Vec<TelemetryRecord>, Box<dyn std::error::Error + Send + Sync>>;
    // async fn save(&self, telemetry: TelemetryRecord) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
} 