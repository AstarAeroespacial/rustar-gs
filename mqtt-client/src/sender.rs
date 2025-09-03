use rumqttc::{AsyncClient, ClientError, EventLoop, MqttOptions, QoS};
use std::time::Duration;
use uuid::Uuid;

pub struct MqttSender {
    client: AsyncClient,
    eventloop: Option<EventLoop>,
}

impl MqttSender {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> (Self, EventLoop) {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        (Self { client, eventloop: None }, eventloop)
    }

    pub fn new_standalone(host: &str, port: u16, keep_alive: Duration) -> Self {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        Self { client, eventloop: Some(eventloop) }
    }

    pub fn from_client(client: AsyncClient, eventloop: Option<EventLoop>) -> Self {
        Self {
            client: client.clone(),
            eventloop: eventloop,
        }
    }

    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }

    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), ClientError> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes())
            .await?;

        println!("Published message {} to topic: {}", payload, topic);
        Ok(())
    }

    pub async fn flush(&mut self) {
        if let Some(evl) = &mut self.eventloop {
            let _ = evl.poll().await;
        }
    }
}
