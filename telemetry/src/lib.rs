use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryRecord {
    id: String,
    timestamp: i64,
    temperature: f32,
    voltage: f32,
    current: f32,
    battery_level: i32,
}
