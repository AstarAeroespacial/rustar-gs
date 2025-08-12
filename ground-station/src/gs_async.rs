use std::time::Duration;

use tokio::{
    pin, select,
    sync::mpsc::{Receiver, Sender, channel},
    task,
    time::{Instant, sleep, sleep_until},
};
use tokio_stream::{Stream, StreamExt};

use crate::{Command, GroundStationStateOrConfigOrWhatever, Message};

pub async fn run(
    // impl stream also?
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
        tx_cmd.send(Command::Ping).await.unwrap();

        // Wait for the Pong message
        let msg = rx_msg.recv().await.unwrap();

        assert_eq!(msg, Message::Pong);
    }
}
