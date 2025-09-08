use std::time::Duration;

use mqtt_client::receiver::MqttReceiver;
use tokio;
use tokio_stream::{self, StreamExt};

#[tokio::main]
async fn main() {
    println!("Running mqtt with broker localhost:8888 subbed to tpic hello-mqtt");
    println!("Send 'close' to close the receiver");
    let mut receiver = MqttReceiver::new("127.0.0.1", 8888, Duration::from_secs(30));
    let _ = receiver.subscribe("hello-mqtt").await;

    loop {
        let msg = receiver.next().await;
        if let Some(msg) = msg {
            println!("Received message: {}", msg);
            if msg == "close".to_string() {
                break;
            }
        }
    }

    receiver.close();
    println!("Receiver closed")
}
