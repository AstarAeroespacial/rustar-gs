use rumqttc::{
    AsyncClient,
    Event::{Incoming, Outgoing},
    EventLoop, MqttOptions,
    Packet::Publish,
    QoS,
};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::Stream;
use uuid::Uuid;

async fn receiving_loop(
    mut eventloop: EventLoop,
    mut close_rx: oneshot::Receiver<()>,
    tx: mpsc::UnboundedSender<String>,
) {
    loop {
        tokio::select! {
            _ = &mut close_rx => {
                // TODO: I think this is not necessary
                break;
            }
            ev = eventloop.poll() => {
                match ev {
                    Ok(Incoming(Publish(msg))) => {
                        if let Ok(s) = String::from_utf8(msg.payload.to_vec()) {
                            let _ = tx.send(s);
                        }
                    }
                    Ok(Incoming(_)) => { /* Other incoming packets */ }
                    Ok(Outgoing(_)) => { /* println!("Outgoing event"); */ }
                    Err(e) => {
                        eprintln!("Eventloop error: {}", e);
                    }
                }
            }
        }
    }
}

pub struct MqttReceiver {
    client: AsyncClient,
    rx: mpsc::UnboundedReceiver<String>,
    close_tx: Option<oneshot::Sender<()>>,
}

impl MqttReceiver {
    pub fn new(host: &str, port: u16, keep_alive: Duration) -> Self {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);
        let (close_tx, close_rx) = oneshot::channel::<()>();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            receiving_loop(eventloop, close_rx, tx).await;
        });

        Self {
            client,
            rx,
            close_tx: Some(close_tx),
        }
    }

    pub fn from_client(client: AsyncClient, eventloop: EventLoop) -> Self {
        let (close_tx, close_rx) = oneshot::channel::<()>();
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            receiving_loop(eventloop, close_rx, tx).await;
        });

        Self {
            client: client.clone(),
            rx,
            close_tx: Some(close_tx),
        }
    }

    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }

    pub fn close(&mut self) {
        if let Some(tx) = self.close_tx.take() {
            let _ = tx.send(());
        }
    }

    pub async fn subscribe(&self, topic: &str) -> Result<(), rumqttc::ClientError> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await?;
        println!("Subscribed to {}", topic);
        Ok(())
    }
}

impl Stream for MqttReceiver {
    type Item = String;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}
