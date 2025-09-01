use rumqttc::{AsyncClient, EventLoop, QoS, MqttOptions};
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct MqttReceiver {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl MqttReceiver {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> Self {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        Self { client, eventloop }
    }

    pub fn from_client(client: AsyncClient, eventloop: EventLoop) -> Self {
        Self { client, eventloop }
    }
    
    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }
    
    pub async fn run(&mut self, mut shutdown: oneshot::Receiver<()>) {
        if let Err(e) = self.client.subscribe("test-topic", QoS::AtLeastOnce).await {
            eprintln!("Error subscribing to topic: {:?}", e)
        } else {
            println!("Subscribed to topic: test-topic")
        }

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    println!("MqttReceiver: shutdown signal received");
                    break;
                }
                event = self.eventloop.poll() => {
                    match event {
                        Ok(notif) => println!("Notif: {:?}", notif),
                        Err(e) => eprintln!("Connection error in recv: {:?}", e)
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Err(e) = self.client.disconnect().await {
            eprintln!("Error disconnecting MQTT client: {:?}", e);
        }
    }
}

