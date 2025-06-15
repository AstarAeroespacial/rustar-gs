use hdlc::{SpecialChars, decode};
use telemetry_protocol::protocol::TelemetryPacket;
use zmq;

const TELEMETRY_SUB_ADDR: &str = "tcp://127.0.0.1:5555";

fn main() {
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    subscriber.connect(TELEMETRY_SUB_ADDR).unwrap();
    subscriber.set_subscribe(b"").unwrap();

    loop {
        println!("Waiting for telemetry data...");
        let frame = subscriber.recv_bytes(0).unwrap();
        let packet_bytes = decode(&frame, SpecialChars::default()).unwrap();
        let packet = TelemetryPacket::from_bytes(&packet_bytes);
        println!("Received (framed) packet: {:?}", packet);
    }
}
