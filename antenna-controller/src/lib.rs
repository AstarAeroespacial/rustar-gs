use serialport::SerialPort;
use std::{io, time::Duration};

pub trait AntennaController {
    /// Sends data to the antenna.
    fn send(
        &mut self,
        azimuth: f64,
        elevation: f64,
        sat_name: &str,
        downlink_number: i64,
    ) -> io::Result<()>;
}

/// A controller for an antenna, allowing communication via a serial port.
pub struct SerialAntennaController {
    pub port: Box<dyn SerialPort>,
}

impl SerialAntennaController {
    /// Creates a new `AntennaController` instance with the specified port name and baud rate.
    pub fn new(port_name: &str, baud_rate: u32) -> io::Result<Self> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;

        Ok(Self { port })
    }
}

impl AntennaController for SerialAntennaController {
    fn send(
        &mut self,
        azimuth: f64,
        elevation: f64,
        sat_name: &str,
        downlink_number: i64,
    ) -> io::Result<()> {
        let data = format!(
            "SN={},AZ={:.1},EL={:.1},DN={}",
            sat_name, azimuth, elevation, downlink_number
        );

        self.port.write_all(data.as_bytes())
    }
}
