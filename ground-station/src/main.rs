mod config;

use crate::config::Config;
use antenna_controller::{self, AntennaController, mock::MockController};
use api::{ApiDoc, add_job, root};
use axum::{
    Router,
    routing::{get, post},
};
use chrono::Utc;
use demod::{Demodulator, example::ExampleDemod};
use framing::{deframer::Deframer, mock_deframer::MockDeframer};
use jobs::JobScheduler;
use mqtt_client::sender::MqttSender;
use packetizer::{Packetizer, packetizer::TelemetryRecordPacketizer};
use sdr::{MockSdr, SdrCommand, sdr_task};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpListener, sync::mpsc};
use tracking::Tracker;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load configuration: {}", err);
        eprintln!("Please create a config.toml file in the current directory.");
        eprintln!("See the example config.toml for the required format.");
        std::process::exit(1);
    });

    println!("Loaded configuration:");
    println!("  MQTT: {}:{}", config.mqtt.host, config.mqtt.port);
    println!(
        "  Ground Station: lat={}, lon={}, alt={}m",
        config.ground_station.latitude,
        config.ground_station.longitude,
        config.ground_station.altitude
    );
    println!("  API: {}:{}", config.api.host, config.api.port);

    let observer = tracking::Observer::new(
        config.ground_station.latitude,
        config.ground_station.longitude,
        config.ground_station.altitude,
    );

    let (_mqtt_send, _eventloop) =
        MqttSender::new(&config.mqtt.host, config.mqtt.port, config.mqtt.timeout());

    // Create channel for sending jobs from API to scheduler
    let (job_tx, mut job_rx) = mpsc::unbounded_channel::<jobs::Job>();
    let mut scheduler = JobScheduler::new();

    let api_addr = format!("{}:{}", config.api.host, config.api.port);
    let listener = TcpListener::bind(&api_addr).await.unwrap();

    let router = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route("/jobs", post(add_job))
        .with_state(job_tx);

    tokio::spawn(async move {
        println!("Swagger UI available at http://{}/docs", api_addr);

        axum::serve(listener, router).await.unwrap();
    });

    loop {
        tokio::select! {
            // Receive jobs from API and add them to scheduler
            Some(job) = job_rx.recv() => {
                println!("Received job for {:?}", job.timestamp);

                if let Err(e) = scheduler.set_job(jobs::ScheduledJob::from_job(job)) {
                    eprintln!("Failed to schedule job: {:?}", e);
                }
            }
            // Execute scheduled jobs
            job = scheduler.next_job() => {
                println!("\nSTARTING PASS\n");

                let observer_clone = observer.clone();

                // Lanzar tracking en background
                tokio::spawn(async move {

                    // INIT SETUP
                    let tracker = Tracker::new(&observer_clone, job.elements).unwrap();
                    let stop = Arc::new(AtomicBool::new(false));

                    let deframer = MockDeframer::new();
                    let demodulator = ExampleDemod::new();
                    let packetizer = TelemetryRecordPacketizer::new();
                    let controller = Arc::new(Mutex::new(MockController));

                    let sdr = MockSdr::new(48_000.0, 1200.0, 512);
                    let (cmd_tx, cmd_rx) = mpsc::channel(1); // tokio channel
                    let (samp_tx, samp_rx) = std::sync::mpsc::channel(); // standard channel
                    // END SETUP

                    let sdr_handle = tokio::spawn(sdr_task(sdr, cmd_rx, samp_tx));

                    // TRACKING
                    let stop_clone = stop.clone();
                    let controller_clone = controller.clone();
                    let tracker_handle = tokio::spawn(async move {
                        for i in 0..5 {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            let obs = tracker.track(Utc::now()).unwrap();

                            println!("Tracking step {}: Az={:.1}°, El={:.1}°",
                            i, obs.azimuth.to_degrees(), obs.elevation.to_degrees());

                            controller_clone
                            .lock()
                            .unwrap()
                                .send(obs.azimuth.to_degrees(), obs.elevation.to_degrees(), "ISS", 145800)
                                .unwrap();

                            // TODO: consider using crate engineering units, might be elegant
                            cmd_tx.send(SdrCommand::SetRxFrequency(435_000_000.0)).await.unwrap();

                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }

                        println!("\nPass ended, stopping SDR and tracker.\n");
                        stop_clone.store(true, Ordering::Relaxed);
                    });

                    // BITS/FRAMES - Move to blocking task to handle std::sync::mpsc
                    let stop_clone = stop.clone();
                    let frame_handle = tokio::task::spawn_blocking(move || {
                        let bits = demodulator.bits(samp_rx.into_iter());
                        let frames = deframer.frames(bits);
                        let mut packets = packetizer.packets(frames);

                        while !stop_clone.load(Ordering::Relaxed) {
                            if let Some(packet) = packets.next() {
                                // TODO: send via MQTT here.
                                dbg!(&packet);
                            }
                        }
                    });

                    let _ = tokio::join!(tracker_handle, sdr_handle, frame_handle);
                });
            }
            msg = mqtt_recv.next() => {
                if let Some(m) = msg {
                    println!("Received command via mqtt: {}", m);
                    match m.trim() {
                        "GET_ELEMENTS" => mqtt_send
                            .publish("sat1/elements", serde_json::to_string(&elements).unwrap().as_str())
                            .await
                            .unwrap(),
                        "GET_OBSERVER" => mqtt_send
                            .publish("sat1/observer", serde_json::to_string(&observer).unwrap().as_str())
                            .await
                            .unwrap(),
                        _ if m.starts_with("SET_OBSERVER=") => {
                            let maybe_observer = m.strip_prefix("SET_OBSERVER=").unwrap();

                            if let Ok(o) = serde_json::from_str(maybe_observer.trim()) {
                                observer = o;
                                mqtt_send.publish("sat1/observer", "OK").await.unwrap();
                            } else {
                                mqtt_send
                                    .publish("sat1/observer", "INVALID OBSERVER")
                                    .await
                                    .unwrap();
                            }
                        }
                        _ if m.starts_with("SET_ELEMENTS=") => {
                            let maybe_elements = m.strip_prefix("SET_ELEMENTS=").unwrap();

                            if let Ok(e) = serde_json::from_str(maybe_elements.trim()) {
                                elements = e;
                                mqtt_send.publish("sat1/elements", "OK").await.unwrap();
                            } else {
                                mqtt_send
                                    .publish("sat1/elements", "INVALID ELEMENTS")
                                    .await
                                    .unwrap();
                            }
                        }
                        _ => mqtt_send
                            .publish("sat1/responses", "INVALID COMMAND")
                            .await
                            .unwrap(),
                    }
                }
            }
        }
    }
}
