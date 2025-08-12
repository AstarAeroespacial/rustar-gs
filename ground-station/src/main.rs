mod gs_async;
mod gs_sync;

use std::time::Duration;

use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::select;
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

#[derive(Debug)]
pub enum CommandError {}

impl TryFrom<Vec<u8>> for Command {
    type Error = CommandError;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        todo!()
    }
}

/// Messages sent from the ground station.
#[derive(Debug, PartialEq)]
enum Message {
    Pong,
}

impl Into<Vec<u8>> for Message {
    fn into(self) -> Vec<u8> {
        todo!()
    }
}

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

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    // TODO: 1. Load/validate config.
    // 2. Create the state.
    let mut state = GroundStationStateOrConfigOrWhatever::default();

    // 3. Set up MQTT.

    let mut mqttoptions = MqttOptions::new("rumqtt-async", "test.mosquitto.org", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    client
        .subscribe("hello/rumqtt", QoS::AtLeastOnce)
        .await
        .unwrap();

    // 4. Set up the TCP socket for connecting with the CLI.

    // 5. Launch the main task.
    tokio::spawn(async move {
        loop {
            select! {
                // Check MQTT.
                Ok(notification) = eventloop.poll() => {
                    // notification
                    if let Event::Incoming(Packet::Publish(publish)) = notification {
                        match Command::try_from(publish.payload.to_vec()).unwrap() {
                            Command::Ping => {
                                // It's cheap, inside it's just a Sender.
                                let client_clone = client.clone();

                                tokio::spawn(async move {
                                    client_clone.publish("antenna/1", QoS::AtLeastOnce, false, Message::Pong).await.unwrap();
                                });
                            },
                            Command::SetTle(tle) => state.update_tle(tle),
                        }
                    }
                }
                // Check TCP socket for CLI input.
                // Check timer.
                // Some(v) = rx_cli.recv() => { dbg!(v) }
            }
        }
    });
}
