use crate::models::responses::TelemetryResponse;
use crate::models::telemetry::TelemetryRecord;
use crate::repository::telemetry::TelemetryRepository;

pub struct TelemetryService {
    repository: TelemetryRepository,
}

impl TelemetryService {
    pub fn new(repository: TelemetryRepository) -> Self {
        Self { repository }
    }

    pub async fn get_latest_telemetry(
        &self,
        sat_name: String,
        limit: i32,
    ) -> Result<Vec<TelemetryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.repository.get_latest(sat_name, limit).await?;
        let responses = records
            .into_iter()
            .map(|record| TelemetryResponse {
                timestamp: record.timestamp,
                temperature: record.temperature,
                voltage: record.voltage,
                current: record.current,
                battery_level: record.battery_level,
            })
            .collect();

        Ok(responses)
    }

    pub async fn get_historic_telemetry(
        &self,
        sat_name: String,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> Result<Vec<TelemetryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.repository.get_historic(sat_name, start_time, end_time).await?;
        let responses = records
            .into_iter()
            .map(|record| TelemetryResponse {
                timestamp: record.timestamp,
                temperature: record.temperature,
                voltage: record.voltage,
                current: record.current,
                battery_level: record.battery_level,
            })
            .collect();

        Ok(responses)
    }

    pub async fn save_telemetry(
        &self,
        timestamp: i64,
        temperature: f32,
        voltage: f32,
        current: f32,
        battery_level: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let record = TelemetryRecord::new(timestamp, temperature, voltage, current, battery_level);
        self.repository.save(record).await
    }
}
