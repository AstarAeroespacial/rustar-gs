use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub ground_station: GroundStationConfig,
    pub api: ApiConfig,
    pub sdr: SdrConfig,
}

/// MQTT Transport Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MqttTransport {
    Tcp,
    Tls,
}

/// MQTT Broker Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub transport: MqttTransport,
    pub timeout_seconds: u64,
    pub auth: Option<MqttAuth>,
}

/// MQTT Authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttAuth {
    pub username: String,
    pub password: String,
}

/// Ground Station Location and Identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStationConfig {
    pub id: String,
    pub location: Location,
}

/// Geographic Location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

/// SDR Type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SdrConfig {
    Mock,
    ZmqMock { zmq_endpoint: String },
    Soapy { soapy_string: String },
}

/// API Server Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
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
