use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub id: String,
    pub timestamp: i64,
    pub temperature: f32,
    pub voltage: f32,
    pub current: f32,
    pub battery_level: i32,
}

impl TelemetryRecord {
    pub fn new(
        timestamp: i64,
        temperature: f32,
        voltage: f32,
        current: f32,
        battery_level: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp,
            temperature,
            voltage,
            current,
            battery_level,
        }
    }

    pub fn with_id(
        id: String,
        timestamp: i64,
        temperature: f32,
        voltage: f32,
        current: f32,
        battery_level: i32,
    ) -> Self {
        Self {
            id,
            timestamp,
            temperature,
            voltage,
            current,
            battery_level,
        }
    }
}
