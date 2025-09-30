use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub ground_station: GroundStationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStationConfig {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

impl Config {
    /// Load configuration from a TOML file
    pub fn load() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            // Load from config.toml file (required)
            .add_source(config::File::with_name("ground-station/config"))
            // Add environment variables with RUSTAR_ prefix
            .add_source(config::Environment::with_prefix("RUSTAR"))
            .build()?;

        settings.try_deserialize()
    }
}

impl MqttConfig {
    /// Get timeout as Duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}
