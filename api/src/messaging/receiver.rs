use crate::{models::telemetry::TelemetryRecord, services::telemetry_service::TelemetryService};
use rumqttc::{
    AsyncClient,
    Event::{self, Incoming, Outgoing},
    EventLoop, MqttOptions,
    Packet::Publish,
    QoS,
};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::sync::oneshot;
use uuid::Uuid;

pub struct MqttReceiver {
    client: AsyncClient,
    eventloop: EventLoop,
    telemetry_service: Arc<TelemetryService>,
}

impl MqttReceiver {
    #[allow(dead_code)]
    pub fn new(
        host: &str,
        port: u16,
        keep_alive: Duration,
        telemetry_service: Arc<TelemetryService>,
    ) -> Self {
        let client_id = format!("rustar-api-{}", Uuid::new_v4());
        let mut options = MqttOptions::new(client_id, host, port);
        options.set_keep_alive(keep_alive);

        let (client, eventloop) = AsyncClient::new(options, 10);

        Self {
            client,
            eventloop,
            telemetry_service,
        }
    }

    pub fn from_client(
        client: AsyncClient,
        eventloop: EventLoop,
        telemetry_service: Arc<TelemetryService>,
    ) -> Self {
        Self {
            client,
            eventloop,
            telemetry_service,
        }
    }

    #[allow(dead_code)]
    pub fn client(&self) -> AsyncClient {
        self.client.clone()
    }

    pub async fn run(&mut self, mut shutdown: oneshot::Receiver<()>) {
        if let Err(e) = self.client.subscribe("test-topic", QoS::AtLeastOnce).await {
            eprintln!("Error subscribing to topic: {:?}", e)
        } else {
            println!("Subscribed to topic: test-topic")
        }

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    println!("MqttReceiver: shutdown signal received");
                    break;
                }
                event = self.eventloop.poll() => {
                    match event {
                        Ok(notif) => {
                            if let Err(e) = self.handle_event(notif).await {
                                eprintln!("Error handling event: {:?}", e);
                            }
                        },
                        Err(e) => eprintln!("Connection error in recv: {:?}", e)
                    }
                }
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        if let Err(e) = self.client.disconnect().await {
            eprintln!("Error disconnecting MQTT client: {:?}", e);
        }
    }

    async fn handle_event(
        &self,
        event: Event,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("Notif: {:?}", event);

        match event {
            Incoming(pk) => {
                println!("Received incoming event: {:?}", pk);

                if let Publish(msg) = pk {
                    let msg_text = String::from_utf8(msg.payload.to_vec());
                    match msg_text {
                        Ok(msg) => {
                            println!("Message received:{:?}", msg);

                            // Parse the message into a map of key-value pairs, treating everything as a string
                            let mut map = std::collections::HashMap::new();
                            for pair in msg.split('|') {
                                let pair = pair.trim();
                                if pair.is_empty() {
                                    continue;
                                }
                                if let Some(idx) = pair.find(':') {
                                    let key = pair[..idx].trim().to_string();
                                    let value = pair[idx + 1..].trim().to_string();
                                    map.insert(key, value);
                                }
                            }
                            let msg_type = map.get("type").map(|s| s.as_str()).unwrap_or("unknown");

                            match msg_type {
                                "telemetry" => {
                                    if let Ok(telemetry) = parse_telemetry(map) {
                                        self.telemetry_service
                                            .save_telemetry(
                                                telemetry.timestamp,
                                                telemetry.temperature,
                                                telemetry.voltage,
                                                telemetry.current,
                                                telemetry.battery_level,
                                            )
                                            .await?;

                                        println!("Telemetry saved: {:?}", telemetry);
                                    } else {
                                        eprintln!("Error saving telemetry");
                                    }
                                }
                                _ => println!("Unknown message type. Message: {:?}", map),
                            }
                        }
                        Err(e) => eprintln!("Error converting payload: {:?}", e),
                    };
                } else {
                    println!("Incoming event: {:?}", pk)
                }
            }
            Outgoing(ev) => println!("Outgoing event: {:?}", ev),
        }

        Ok(())
    }
}

fn parse_telemetry(
    map: HashMap<String, String>,
) -> Result<TelemetryRecord, Box<dyn std::error::Error + Send + Sync>> {
    let timestamp = map
        .get("timestamp")
        .ok_or("Timestamp not found")?
        .parse::<i64>()?;
    let temperature = map
        .get("temperature")
        .ok_or("Temperature not found")?
        .parse::<f32>()?;
    let voltage = map
        .get("voltage")
        .ok_or("Voltage not found")?
        .parse::<f32>()?;
    let current = map
        .get("current")
        .ok_or("Current not found")?
        .parse::<f32>()?;
    let battery_level = map
        .get("battery_level")
        .ok_or("Battery level not found")?
        .parse::<i32>()?;
    Ok(TelemetryRecord::new(
        timestamp,
        temperature,
        voltage,
        current,
        battery_level,
    ))
}
