use rumqttc::{MqttOptions, AsyncClient, QoS, ClientError, EventLoop};
use std::time::Duration;
use std::error::Error;
use uuid::Uuid;
pub struct MqttBroker {
    client: AsyncClient,
    // eventloop: EventLoop,
}

impl MqttBroker {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> (Self, EventLoop) {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        (Self { client }, eventloop)
    }
    
    pub async fn publish(&self, topic: &str, payload: &str) -> Result<(), ClientError> {
        self.client.publish(topic, QoS::AtLeastOnce, false, payload.as_bytes()).await?;
        // self.eventloop.poll().await;
        println!("Published message {} to topic: {}", payload, topic);
        Ok(())
    }
}
