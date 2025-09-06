use crate::messaging::broker::MqttBroker;
use rumqttc::ClientError;

pub struct MessageService {
    pub message_broker: MqttBroker,
}

impl MessageService {
    pub fn new(message_broker: MqttBroker) -> Self {
        Self { message_broker }
    }

    pub async fn send_message(&self, topic: &str, payload: &str) -> Result<(), ClientError> {
        self.message_broker.publish(topic, payload).await
    }
}
