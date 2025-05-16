use chrono::{DateTime, Utc};
use std::{
    default,
    fs::File,
    io::{self, BufRead, Cursor, Read},
    path::Path,
    str::FromStr,
};

// TelemetryPacket (16 bytes)
#[derive(Debug)]
pub struct TelemetryPacket {
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

impl TelemetryPacket {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(16);
        buf.push(self.pkt_type);

        // Payload length as u16
        buf.extend_from_slice(&self.length.to_be_bytes());

        // Payload
        let payload_bytes = self.payload.to_bytes();
        buf.extend_from_slice(&payload_bytes);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);

        // Read pkt_type (1 byte)
        let mut pkt_type_bytes = [0u8];
        cursor.read_exact(&mut pkt_type_bytes).unwrap();
        let pkt_type = u8::from_be_bytes(pkt_type_bytes);

        // Read length (2 bytes)
        let mut length_bytes = [0u8; 2];
        cursor.read_exact(&mut length_bytes).unwrap();
        let length = u16::from_be_bytes(length_bytes);

        // Read payload
        let mut payload_bytes = vec![0u8; length as usize];
        cursor.read_exact(&mut payload_bytes).unwrap();

        let payload = TelemetryData::from_bytes(&payload_bytes);

        TelemetryPacket {
            pkt_type,
            length,
            payload,
        }
    }
}

// TelemetryData (10 bytes)
#[derive(Debug)]
pub struct TelemetryData {
    pub timestamp: u32,  // seconds since UNIX epoch
    pub temp: f32,       // degrees Celsius
    pub volt: f32,       // millivolts
    pub curr: f32,       // milliamps
    pub battery_soc: u8, // percentage
}

impl TelemetryData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(10);
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

// Function to parse a line of telemetry data
fn parse_line(line: &str) -> Option<TelemetryPacket> {
    let mut time: Option<DateTime<Utc>> = None;
    let mut temp: Option<f32> = None;
    let mut volt: Option<f32> = None;
    let mut curr: Option<f32> = None;
    let mut battery_soc: Option<u8> = None;

    for part in line.split(';') {
        let kv: Vec<&str> = part.split('=').collect();
        if kv.len() == 2 {
            match kv[0] {
                "TIME" => time = Some(DateTime::from_str(kv[1]).ok()?),
                "TEMP" => temp = Some(kv[1].parse().ok()?),
                "VOLT" => volt = Some(kv[1].parse().ok()?),
                "CURR" => curr = Some(kv[1].parse().ok()?),
                "BATTERY_SOC" => battery_soc = Some(kv[1].parse().ok()?),
                _ => {}
            }
        }
    }

    if let (Some(time), Some(temp), Some(volt), Some(curr), Some(battery_soc)) =
        (time, temp, volt, curr, battery_soc)
    {
        let timestamp = time.timestamp() as u32;
        let frame = TelemetryData {
            timestamp,
            temp,
            volt,
            curr,
            battery_soc,
        };

        let length = frame.to_bytes().len() as u16;
        Some(TelemetryPacket {
            pkt_type: 0x01,
            length,
            payload: frame,
        })
    } else {
        None
    }
}

fn main() {
    let path = Path::new("telemetry.txt");
    let mut telemetry_packet: TelemetryPacket = TelemetryPacket::default();

    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(line_str) = line {
                if let Some(packet) = parse_line(&line_str) {
                    telemetry_packet = packet;
                    println!("Parsed packet: {:?}\n", telemetry_packet);
                } else {
                    println!("Failed to parse line: {}", line_str);
                }
            }
        }
    };

    // encoding telemetry packet to bytes
    let encoded_packet = telemetry_packet.to_bytes();
    println!("Encoded packet: {:?}\n", encoded_packet);

    // decoding telemetry packet from bytes
    let decoded_packet = TelemetryPacket::from_bytes(&encoded_packet);
    println!("Decoded packet: {:?}\n", decoded_packet);

    // Check if the decoded packet matches the original packet
    assert_eq!(telemetry_packet.pkt_type, decoded_packet.pkt_type);
    assert_eq!(telemetry_packet.length, decoded_packet.length);
    assert_eq!(
        telemetry_packet.payload.timestamp,
        decoded_packet.payload.timestamp
    );
    assert_eq!(telemetry_packet.payload.temp, decoded_packet.payload.temp);
    assert_eq!(telemetry_packet.payload.volt, decoded_packet.payload.volt);
}
