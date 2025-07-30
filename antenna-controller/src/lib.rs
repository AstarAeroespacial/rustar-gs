use serialport::SerialPort;
use std::{io, time::Duration};

/// A controller for an antenna, allowing communication via a serial port.
pub struct AntennaController {
    pub port: Box<dyn SerialPort>,
}

impl AntennaController {
    /// Creates a new `AntennaController` instance with the specified port name and baud rate.
    pub fn new(port_name: &str, baud_rate: u32) -> io::Result<Self> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;
        Ok(AntennaController { port })
    }

    /// Sends data to the antenna controller.
    pub fn send(&mut self, data: &[u8]) -> io::Result<()> {
        self.port.write_all(data)
    }
}
