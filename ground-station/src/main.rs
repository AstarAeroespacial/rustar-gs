use std::time::Duration;

use tokio::{
    pin, select,
    sync::mpsc::{Receiver, Sender, channel},
    task,
    time::{Instant, sleep, sleep_until},
};
use tokio_stream::{Stream, StreamExt};

// /// Anything that can send a [`Command`] to the ground station.
// /// Could be reading from an MQTT topic, `stdin`, or whatever.
// pub trait Controller {
//     async fn next_command(&mut self) -> Option<Command>;
// }

/// A command (e.g. control or configuration) sent to the ground station.
#[derive(Debug)]
pub enum Command {
    Ping,
}

/// Messages sent from the ground station.
#[derive(Debug, PartialEq)]
enum Message {
    Pong,
}

struct Passes {
    next_pass: Instant,
}

impl Passes {
    /// Returns the next scheduled pass of the satellite.
    pub fn next_pass(&self) -> Instant {
        todo!()
    }

    // pub fn recalculate(&self) {
    //     todo!()
    // }
}

// impl Stream for Passes {
//     type Item = Instant;

//     fn poll_next(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
//         todo!()
//     }
// }

async fn run(
    mut controller: Receiver<Command>,
    publisher: Sender<Message>,
    mut passes: impl Iterator<Item = Instant>,
    // mut passes: impl Stream<Item = Instant> + Unpin,
) {
    // Keep a pinned Sleep

    let sleep = sleep_until(passes.next().unwrap());
    tokio::pin!(sleep);

    loop {
        select! {
            // React to ground station commands.
            Some(cmd) = controller.recv() => {
                let publisher = publisher.clone();

                tokio::spawn(async move {
                    match cmd {
                        Command::Ping => publisher.send(Message::Pong).await.unwrap()
                    }
                });
            }
            // Wait for the pass.
            _ = &mut sleep => {
            // Some(_) = passes.next() => {
                let handle = tokio::spawn(track());
                handle.await.unwrap();

                println!("Updating pass future...");
                sleep.as_mut().reset(passes.next().unwrap());
            }
        }
    }
}

async fn track() {
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
    use tokio::sync::mpsc;
    use tokio_stream::{self, StreamExt};

    #[tokio::test]
    async fn test_ping_pong() {
        let (tx_cmd, rx_cmd) = mpsc::channel(1);
        let (tx_msg, mut rx_msg) = mpsc::channel(1);

        // let stream = tokio_stream::iter(vec![Instant::now() + Duration::from_secs(200)]);
        // stream.next()

        // Spawn the run function as a background task
        tokio::spawn(run(rx_cmd, tx_msg, stream));

        // Send a Ping command
        tx_cmd.send(Command::Ping).await.expect("send ping");

        // Wait for the Pong message
        let msg = rx_msg.recv().await.expect("receive pong");

        assert_eq!(msg, Message::Pong);
    }
}
