use std::time::Duration;

use mqtt_client::sender::MqttSender;

#[tokio::main]
async fn main() {
    println!("Running mqtt with broker localhost:8888 at topic hello-mqtt");
    let mut sender = MqttSender::new_standalone("127.0.0.1", 8888, Duration::from_secs(30));

    for i in 0..7 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        sender
            .publish(
                "hello-mqtt",
                format!("Hello! This is message n{}", i + 1).as_str(),
            )
            .await
            .expect("Error sending message. Is the broker on?");
        sender.flush().await;
    }
}
