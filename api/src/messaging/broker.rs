use rumqttc::{AsyncClient, ClientError, EventLoop, MqttOptions, QoS};
use std::time::Duration;
use uuid::Uuid;

pub struct MqttBroker {
    client: AsyncClient,
}

impl MqttBroker {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> (Self, EventLoop) {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        (Self { client }, eventloop)
    }

    #[allow(dead_code)]
    pub fn from_client(client: AsyncClient) -> Self {
        Self {
            client: client.clone(),
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
}
