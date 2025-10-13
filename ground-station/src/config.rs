use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub ground_station: GroundStationConfig,
    pub api: ApiConfig,
    pub sdr: SdrConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MqttConfig {
    pub host: String,
    pub port: u16,
    pub transport: String,
    pub timeout_seconds: u64,
    pub username: String,
    pub password: String,
}

impl MqttConfig {
    pub fn validate(&self) -> Result<(), String> {
        match self.transport.as_str() {
            "tcp" | "tls" => Ok(()),
            _ => Err(format!(
                "Unsupported MQTT transport: '{}'. Use 'tcp' or 'tls'",
                self.transport
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundStationConfig {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdrConfig {
    pub r#type: String,
    pub zmq_endpoint: Option<String>,
}

impl SdrConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.r#type == "zmq_mock" && self.zmq_endpoint.is_none() {
            return Err("if the type is zmq_mock, include zmq_endpoint".to_string());
        }
        Ok(())
    }
}

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
