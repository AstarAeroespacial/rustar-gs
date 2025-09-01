use rumqttc::{
    AsyncClient,
    Event::{self, Incoming, Outgoing},
    EventLoop, MqttOptions,
    Packet::Publish,
    QoS,
};
use std::time::Duration;
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct MqttReceiver {
    client: AsyncClient,
    eventloop: EventLoop,
}

impl MqttReceiver {
    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
                        Ok(notif) => self.handle_event(notif),
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

    fn handle_event(&self, event: Event) {
        println!("Notif: {:?}", event);

        match event {
            Incoming(pk) => {
                println!("Received incoming event: {:?}", pk);

                if let Publish(msg) = pk {
                    let msg_text = String::from_utf8(msg.payload.to_vec());
                    match msg_text {
                        Ok(msg) => println!("Message received:{:?}", msg),
                        Err(e) => eprintln!("Error converting payload: {:?}", e),
                    };
                } else {
                    println!("Incoming event: {:?}", pk)
                }
            }
            Outgoing(ev) => println!("Outgoing event: {:?}", ev),
        }
    }
}
