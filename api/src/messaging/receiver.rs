use rumqttc::{AsyncClient, EventLoop, QoS, MqttOptions};
use std::{sync::{atomic::{AtomicBool, Ordering}, Arc}, time::Duration};
use uuid::Uuid;

pub struct MqttReceiver {
    client: AsyncClient,
    eventloop: EventLoop,
    running: AtomicBool,
}

impl MqttReceiver {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> Self {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        Self { client, eventloop, running: AtomicBool::new(true) }
    }

    pub fn from_client(client: AsyncClient, eventloop: EventLoop) -> Self {
        Self { client, eventloop, running: AtomicBool::new(true) }
    }
    
    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }
    
    pub async fn run(&mut self) {
        if let Err(e) = self.client.subscribe("test-topic", QoS::AtLeastOnce).await {
            eprintln!("Error subscribing to topic: {:?}", e)
        } else {
            println!("Subscribed to topic: test-topic")
        }

        loop {
            let event = self.eventloop.poll().await;

            match event {
                Ok(notif) => println!("Notif: {:?}", notif),
                Err(e) => eprintln!("Connection error in recv: {:?}", e)
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }

    pub fn stop (&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

