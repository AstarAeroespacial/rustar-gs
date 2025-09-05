use rumqttc::{
    AsyncClient,
    Event::{Incoming, Outgoing},
    EventLoop, MqttOptions,
    Packet::Publish,
};
use std::{task::Poll, time::Duration};
use tokio_stream::Stream;
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
}

impl Stream for MqttReceiver {
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut poll_fut = Box::pin((&mut self).eventloop.poll());

        match poll_fut.as_mut().poll(cx) {
            Poll::Ready(event) => {
                if let Err(e) = event {
                    eprintln!("Error serializing message: {}", e);
                    return Poll::Pending;
                }
                let event = event.unwrap();

                match event {
                    Incoming(pk) => {
                        println!("Received incoming event: {:?}", pk);

                        if let Publish(msg) = pk {
                            let msg_text = String::from_utf8(msg.payload.to_vec());
                            match msg_text {
                                Ok(msg) => {
                                    println!("Message received:{:?}", msg);

                                    Poll::Ready(Some(msg))
                                }
                                Err(e) => {
                                    eprintln!("Error converting payload: {:?}", e);
                                    Poll::Pending
                                }
                            }
                        } else {
                            println!("Incoming event: {:?}", pk);
                            Poll::Pending
                        }
                    }
                    Outgoing(ev) => {
                        println!("Outgoing event: {:?}", ev);
                        Poll::Pending
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
