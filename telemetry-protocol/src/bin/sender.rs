use chrono::{DateTime, Utc};
use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
    str::FromStr,
    thread, time,
};
use telemetry_protocol::protocol::{TelemetryData, TelemetryPacket};
use zmq;

const TELEMETRY_FILE: &str = "telemetry.txt";
const TELEMETRY_PUB_ADDR: &str = "tcp://127.0.0.1:5555";

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
        let frame = TelemetryData::new(timestamp, temp, volt, curr, battery_soc);

        let length = frame.to_bytes().len() as u16;
        Some(TelemetryPacket::new(0x01, length, frame))
    } else {
        None
    }
}

fn main() {
    let path = Path::new(TELEMETRY_FILE);
    let context = zmq::Context::new();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher.bind(TELEMETRY_PUB_ADDR).unwrap();

    let file = File::open(path).unwrap();
    let reader = io::BufReader::new(file);

    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    loop {
        for line_str in &lines {
            if let Some(packet) = parse_line(line_str) {
                let encoded_packet = packet.to_bytes();
                publisher.send(encoded_packet, 0).unwrap();
                println!("Sent packet: {:?}", packet);
            } else {
                println!("Failed to parse line: {}", line_str);
            }

            thread::sleep(time::Duration::from_secs(1));
        }
    }
}
