use crate::messaging::broker::MqttBroker;
use rumqttc::ClientError;
use std::ops::{Deref, DerefMut};

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

// impl DerefMut for MessageService {
//     fn deref_mut(&mut self) -> &mut MqttBroker {
//         &mut self.message_broker
//     }
// }

// impl Deref for MessageService {
//     type Target = MqttBroker;

//     fn deref(&self) -> &Self::Target {
//         &self.message_broker
//     }
// }
