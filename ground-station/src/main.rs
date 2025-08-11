use std::time::Duration;

use tokio::{
    pin, select,
    sync::mpsc::{Receiver, Sender, channel},
    task,
    time::{Instant, sleep, sleep_until},
};
use tokio_stream::{Stream, StreamExt};
use tracking;

type Tle = String;

// /// Anything that can send a [`Command`] to the ground station.
// /// Could be reading from an MQTT topic, `stdin`, or whatever.
// pub trait Controller {
//     async fn next_command(&mut self) -> Option<Command>;
// }

/// A command (e.g. control or configuration) sent to the ground station.
#[derive(Debug)]
pub enum Command {
    Ping,
    SetTle(Tle),
}

/// Messages sent from the ground station.
#[derive(Debug, PartialEq)]
enum Message {
    Pong,
}

// impl Stream for Passes {
//     type Item = Instant;

//     fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         todo!()
//     }
// }

struct GroundStationStateOrConfigOrWhatever {
    tle: Option<Tle>,
    location: Option<tracking::Observer>,
}

impl GroundStationStateOrConfigOrWhatever {
    pub fn update_tle(&mut self, new_tle: Tle) {
        self.tle = Some(new_tle);
    }
}

impl Default for GroundStationStateOrConfigOrWhatever {
    fn default() -> Self {
        Self {
            tle: Default::default(),
            location: Default::default(),
        }
    }
}

async fn run(
    mut controller: Receiver<Command>,
    publisher: Sender<Message>,
    mut passes: impl Stream<Item = Instant> + Unpin,
    state: &mut GroundStationStateOrConfigOrWhatever,
) {
    loop {
        select! {
            // React to ground station commands.
            Some(cmd) = controller.recv() => {
                match cmd {
                    Command::Ping => {
                        let publisher = publisher.clone();

                        tokio::spawn(async move {
                            publisher.send(Message::Pong).await.unwrap();
                        });
                    }
                    Command::SetTle(tle) => {
                        state.update_tle(tle); // TODO: ack change?
                    }
                }
            }
            // Wait for the pass.
            Some(_) = passes.next() => {
                if let Some(observer) = state.location.as_ref() {
                    let observer_clone = observer.clone();

                    let handle = tokio::spawn(async move {
                        track(&observer_clone).await
                    });

                    handle.await.unwrap();
                }
            }
        }
    }
}

// block on to make sync and bridge the gap?
async fn track(observer: &tracking::Observer) {
    dbg!("Tracking satellite...");
    sleep(Duration::from_secs(10)).await;
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // let (tx_mqtt, rx_mqtt) = channel(1);
    // let (tx_cli, rx_cli) = channel(1);

    // loop {
    //     select! {
    //         Some(v) = rx_mqtt.recv() => { dbg!(v) }
    //         Some(v) = rx_cli.recv() => { dbg!(v) }
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;
    use tokio::sync::mpsc;
    use tokio_stream::{self, StreamExt};

    #[tokio::test]
    async fn test_ping_pong() {
        let (tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_msg, mut rx_msg) = mpsc::channel(1);

        // let stream = tokio_stream::iter(vec![Instant::now() + Duration::from_secs(200)]);
        // stream.next()

        // Spawn the run function as a background task
        tokio::spawn(async move {
            let mut state = GroundStationStateOrConfigOrWhatever::default();

            run(rx_cmd, tx_msg, stream::empty(), &mut state).await;
        });

        // Send a Ping command
        tx_cmd.send(Command::Ping).await.expect("send ping");

        // Wait for the Pong message
        let msg = rx_msg.recv().await.expect("receive pong");

        assert_eq!(msg, Message::Pong);
    }
}
