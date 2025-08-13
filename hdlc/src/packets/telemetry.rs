use crate::frame::{Byte, DeframingError};
use std::{
    default,
    io::{Cursor, Read},
};

// TelemetryPacket (20 bytes)
#[derive(Debug, PartialEq)]
pub(crate) struct TelemetryPacket {
    pub pkt_type: u8, // 0x01
    pub length: u16,  // length of payload
    pub payload: TelemetryData,
}

impl default::Default for TelemetryPacket {
    fn default() -> Self {
        TelemetryPacket {
            pkt_type: 0,
            length: 0,
            payload: TelemetryData {
                timestamp: 0,
                temp: 0.0,
                volt: 0.0,
                curr: 0.0,
                battery_soc: 0,
            },
        }
    }
}

impl TryFrom<Vec<Byte>> for TelemetryPacket {
    type Error = DeframingError;

    fn try_from(info: Vec<Byte>) -> Result<Self, Self::Error> {
        if info.len() < 3 {
            // At least need pkt_type (1) + length (2)
            return Err(DeframingError::InvalidPacketLength);
        }

        // Read pkt_type (1 byte)
        let pkt_type = info[0];

        // Read length (2 bytes)
        let length = u16::from_be_bytes([info[1], info[2]]);

        // Check if we have enough bytes for the payload
        if info.len() != 3 + length as usize {
            return Err(DeframingError::PacketLengthMismatch);
        }

        // Get payload slice
        let payload_bytes = &info[3..3 + length as usize];
        let payload = TelemetryData::from_bytes(payload_bytes);

        Ok(TelemetryPacket {
            pkt_type,
            length,
            payload,
        })
    }
}

// TelemetryData (17 bytes)
#[derive(Debug, PartialEq, Clone)]
pub struct TelemetryData {
    pub timestamp: u32,  // seconds since UNIX epoch
    pub temp: f32,       // degrees Celsius
    pub volt: f32,       // millivolts
    pub curr: f32,       // milliamps
    pub battery_soc: u8, // percentage
}

impl TelemetryData {
    pub fn new(timestamp: u32, temp: f32, volt: f32, curr: f32, battery_soc: u8) -> TelemetryData {
        TelemetryData {
            timestamp,
            temp,
            volt,
            curr,
            battery_soc,
        }
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(17);
        buf.extend_from_slice(&self.timestamp.to_be_bytes());
        buf.extend_from_slice(&self.temp.to_be_bytes());
        buf.extend_from_slice(&self.volt.to_be_bytes());
        buf.extend_from_slice(&self.curr.to_be_bytes());
        buf.push(self.battery_soc);

        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);

        // Read timestamp (4 bytes)
        let mut timestamp_bytes = [0u8; 4];
        cursor.read_exact(&mut timestamp_bytes).unwrap();
        let timestamp = u32::from_be_bytes(timestamp_bytes);

        // Read temp (4 bytes)
        let mut temp_bytes = [0u8; 4];
        cursor.read_exact(&mut temp_bytes).unwrap();
        let temp = f32::from_be_bytes(temp_bytes);

        // Read volt
        let mut volt_bytes = [0u8; 4];
        cursor.read_exact(&mut volt_bytes).unwrap();
        let volt = f32::from_be_bytes(volt_bytes);

        // Read current
        let mut curr_bytes = [0u8; 4];
        cursor.read_exact(&mut curr_bytes).unwrap();
        let curr = f32::from_be_bytes(curr_bytes);

        // Read battery_soc (1 byte)
        let mut battery_soc_bytes = [0u8];
        cursor.read_exact(&mut battery_soc_bytes).unwrap();
        let battery_soc = battery_soc_bytes[0];

        TelemetryData {
            timestamp,
            temp,
            volt,
            curr,
            battery_soc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_data_to_bytes_and_from_bytes() {
        let data = TelemetryData::new(123456, 25.5, 3.7, 1.2, 85);
        let bytes = data.to_bytes();
        let data2 = TelemetryData::from_bytes(&bytes);

        assert_eq!(data.timestamp, data2.timestamp);
        assert_eq!(data.temp, data2.temp);
        assert_eq!(data.volt, data2.volt);
        assert_eq!(data.curr, data2.curr);
        assert_eq!(data.battery_soc, data2.battery_soc);
    }
}
