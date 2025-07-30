use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
// use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TelemetryRecord {
    pub id: String,
    pub time: DateTime<Utc>,
    pub temperature: f64,
    pub voltage: f64,
    pub current: f64,
    pub battery_level: i32,
    pub created_at: DateTime<Utc>,
}

impl TelemetryRecord {
    // pub fn new(temperature: f64, voltage: f64, current: f64, battery_level: i32) -> Self {
    //     Self {
    //         id: Uuid::new_v4().to_string(),
    //         time: Utc::now(),
    //         temperature,
    //         voltage,
    //         current,
    //         battery_level,
    //         created_at: Utc::now(),
    //     }
    // }
} 