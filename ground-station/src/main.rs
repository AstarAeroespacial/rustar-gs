// mod gs_async;
// mod gs_sync;
mod finite_interval;
mod scheduler;

use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::{select, time::Instant};
use tokio_stream::StreamExt;
use tracking::{self, Tracker};

use crate::{
    finite_interval::{FiniteInterval, RealTimeStreamExt},
    scheduler::Scheduler,
};

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

fn datetime_to_instant(dt: DateTime<Utc>) -> Instant {
    let now = Utc::now();
    let dur = (dt - now).to_std().expect("DateTime must be in the future");

    Instant::now() + dur
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

    let observer = tracking::Observer::new(-34.6, -58.4, 2.5);
    let elements = sgp4::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25186.50618345  .00006730  00000+0  12412-3 0  9992".as_bytes(),
        "2 25544  51.6343 216.2777 0002492 336.9059  23.1817 15.50384048518002".as_bytes(),
    )
    .unwrap();

    let scheduler = Scheduler::new();
    tokio::pin!(scheduler);

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
            // Check scheduled events.
            Some(event) = scheduler.next() => {
                match event {
                    scheduler::Event::Pass(pass) => {
                        let start = Instant::now() + Duration::from_secs(2);
                        let end = start + Duration::from_secs(10);
                        let period = Duration::from_secs(3);

                        let tracker = Tracker::new(&observer, &elements).unwrap();

                        let mut observations_stream = FiniteInterval::new(start, end, period).map(|_| Utc::now()).track_with(tracker);

                        while let Some(obs_maybe) = observations_stream.next().await {
                            match obs_maybe {
                                Ok(observation) => {
                                    dbg!(&observation);
                                },
                                Err(_) => todo!(),
                            }
                        }
                    },
                    // TODO: make the pass scheduler to automatically update
                    scheduler::Event::Retry => todo!(),
                }
            }
            // Check TCP socket for CLI input.
            // Check timer.
            // Some(v) = rx_cli.recv() => { dbg!(v) }
        }
    }
}
