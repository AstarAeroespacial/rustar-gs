use crate::repository::TelemetryRepository;
// use crate::models::telemetry::TelemetryRecord;
use crate::models::responses::TelemetryResponse;

pub struct TelemetryService<R>
where
    R: TelemetryRepository,
{
    repository: R,
}

impl<R> TelemetryService<R>
where
    R: TelemetryRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn get_latest_telemetry(&self, limit: i32) -> Result<Vec<TelemetryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.repository.get_latest(limit).await?;
        let responses = records.into_iter().map(|record| TelemetryResponse {
            time: record.time.to_rfc3339(),
            temperature: record.temperature,
            voltage: record.voltage,
            current: record.current,
            battery_level: record.battery_level,
        }).collect();
        
        Ok(responses)
    }

    pub async fn get_historic_telemetry(&self, start_time: Option<i64>, end_time: Option<i64>) -> Result<Vec<TelemetryResponse>, Box<dyn std::error::Error + Send + Sync>> {
        let records = self.repository.get_historic(start_time, end_time).await?;
        let responses = records.into_iter().map(|record| TelemetryResponse {
            time: record.time.to_rfc3339(),
            temperature: record.temperature,
            voltage: record.voltage,
            current: record.current,
            battery_level: record.battery_level,
        }).collect();
        
        Ok(responses)
    }

    // pub async fn save_telemetry(&self, temperature: f64, voltage: f64, current: f64, battery_level: i32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    //     let record = TelemetryRecord::new(temperature, voltage, current, battery_level);
    //     self.repository.save(record).await
    // }
} 