use std::time::Duration;

use mqtt_client::{receiver::MqttReceiver, sender::MqttSender};
use tokio_stream::{self, StreamExt};

async fn listen_mqtt(mut receiver: MqttReceiver) {
    receiver
        .subscribe("hello-mqtt")
        .await
        .expect("Error connecting to mqtt. Is the broker on?");
    for _ in 0..7 {
        let msg = receiver.next().await;
        if let Some(msg) = msg {
            println!("Received message: {}", msg)
        }
    }
}

async fn send_messages(sender: MqttSender) {
    for i in 0..7 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        sender
            .publish(
                "hello-mqtt",
                format!("Hello! This is message n{}", i + 1).as_str(),
            )
            .await
            .expect("Error sending message. Is the broker on?");
    }
}

#[tokio::main]
async fn main() {
    println!("Running mqtt with broker localhost:8888");
    let (sender, eventloop) = MqttSender::new("127.0.0.1", 8888, Duration::from_secs(30));
    let receiver = MqttReceiver::from_client(sender.client(), eventloop);

    // let sender = MqttSender::new_standalone("127.0.0.1", 8888, Duration::from_secs(30));
    // let receiver = MqttReceiver::new("127.0.0.1", 8888, Duration::from_secs(30));

    tokio::join!(send_messages(sender), listen_mqtt(receiver));
}
