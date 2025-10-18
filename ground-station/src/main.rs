mod config;

use crate::config::Config;
use antenna_controller::{self, AntennaController, mock::MockController};
use api::{ApiDoc, add_job, root};
use axum::{
    Router,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use demod::{Demodulator, example::ExampleDemod};
use framing::{deframer::Deframer, mock_deframer::MockDeframer};
use jobs::JobScheduler;
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport, tokio_rustls};
use sdr::{MockSdr, SdrCommand, sdr_task};
use serde::Serialize;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{net::TcpListener, sync::mpsc};
use tokio_rustls::rustls::ClientConfig;
use tracking::Tracker;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

fn create_sdr(sdr_config: &config::SdrConfig) -> Box<dyn sdr::Sdr + Send> {
    match sdr_config {
        config::SdrConfig::Mock => {
            println!("[SDR] Creating Mock SDR");
            Box::new(MockSdr::new(48_000.0, 1200.0, 512))
        }
        config::SdrConfig::ZmqMock { zmq_endpoint } => {
            println!("[SDR] Creating ZMQ Mock SDR: {}", zmq_endpoint);
            Box::new(sdr::ZmqMockSdr::new(zmq_endpoint.clone()))
        }
        config::SdrConfig::Soapy { soapy_string } => {
            println!("[SDR] Creating SoapySDR: {}", soapy_string);
            panic!("SoapySDR not yet implemented");
        }
    }
}

#[tokio::main]
async fn main() {
    // Load configuration
    let config = Config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load configuration: {}", err);
        eprintln!("Please create a config.toml file in the current directory.");
        eprintln!("See the example config.toml for the required format.");
        std::process::exit(1);
    });
    let config = Arc::new(config);

    println!("Loaded configuration:");
    let auth_info = config
        .mqtt
        .auth
        .as_ref()
        .map(|a| format!("username: {}, password: {}", a.username, a.password))
        .unwrap_or_else(|| "no auth".to_string());
    println!(
        "  MQTT: {}:{} ({:?}), {}",
        &config.mqtt.host, &config.mqtt.port, &config.mqtt.transport, auth_info
    );
    println!(
        "  Ground Station: id={}, lat={}, lon={}, alt={}m",
        &config.ground_station.id,
        &config.ground_station.location.latitude,
        &config.ground_station.location.longitude,
        &config.ground_station.location.altitude
    );
    println!("  API: {}:{}", config.api.host, config.api.port);
    println!("  SDR: {:?}", config.sdr);

    let observer = tracking::Observer::new(
        config.ground_station.location.latitude,
        config.ground_station.location.longitude,
        config.ground_station.location.altitude,
    );

    let mut mqttoptions = MqttOptions::new(
        &config.ground_station.id,
        &config.mqtt.host,
        config.mqtt.port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(config.mqtt.timeout_seconds));

    if let Some(ref auth) = config.mqtt.auth {
        mqttoptions.set_credentials(&auth.username, &auth.password);
    }

    match config.mqtt.transport {
        config::MqttTransport::Tls => {
            let mut root_cert_store = tokio_rustls::rustls::RootCertStore::empty();
            root_cert_store.add_parsable_certificates(
                rustls_native_certs::load_native_certs().expect("could not load platform certs"),
            );

            let client_config = ClientConfig::builder()
                .with_root_certificates(root_cert_store)
                .with_no_client_auth();

            mqttoptions.set_transport(Transport::tls_with_config(client_config.into()));
        }
        config::MqttTransport::Tcp => {
            // Default transport (no action needed)
        }
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

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

                let config_clone = config.clone();
                let observer_clone = observer.clone();
                let sdr = create_sdr(&config_clone.sdr);
                let client_clone = client.clone();
                let gs_id_clone = config_clone.ground_station.id.clone();

                // Lanzar tracking en background
                tokio::spawn(async move {

                    // INIT SETUP
                    let tracker = Tracker::new(&observer_clone, job.elements).unwrap();
                    let stop = Arc::new(AtomicBool::new(false));

                    let deframer = MockDeframer::new("IN A HOLE IN THE GROUND".as_bytes().to_vec());
                    let demodulator = ExampleDemod::new();
                    let controller = Arc::new(Mutex::new(MockController));

                    let (cmd_tx, cmd_rx) = mpsc::channel(1); // tokio channel
                    let (samp_tx, samp_rx) = std::sync::mpsc::channel(); // standard channel
                    // END SETUP

                    let sdr_handle = tokio::spawn(sdr_task(sdr, cmd_rx, samp_tx));

                    // TRACKING
                    let stop_clone = stop.clone();
                    let controller_clone = controller.clone();
                    let tracker_handle = tokio::spawn(async move {
                        // TODO: until los in job
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
                    let satellite_name = job.satellite_name.clone();
                    let (frame_tx, mut frame_rx) = mpsc::unbounded_channel();

                    let frame_handle = tokio::task::spawn_blocking(move || {
                        let bits = demodulator.bits(samp_rx.into_iter());
                        let mut frames = deframer.frames(bits);

                        while !stop_clone.load(Ordering::Relaxed) {
                            if let Some(payload) = frames.next().and_then(|frame| frame.info) {
                                frame_tx.send(payload).unwrap();
                            }
                        }
                    });

                    // NOTE: it really is a pita to have both sync and async mixed contexts.
                    // TODO: we should finish moving the demodulator and deframer to be async and be done with it.

                    // MQTT publisher task
                    let mqtt_handle = tokio::spawn(async move {
                        while let Some(payload) = frame_rx.recv().await {
                            let msg = TelemetryMessage::new(gs_id_clone.clone(), Utc::now(), payload);

                            client_clone
                                .publish(
                                    &format!("satellite/{}/telemetry", satellite_name),
                                    QoS::AtLeastOnce,
                                    false,
                                    serde_json::to_string(&msg).unwrap().as_bytes(),
                                )
                                .await
                                .unwrap();
                        }
                    });


                    let _ = tokio::join!(tracker_handle, sdr_handle, frame_handle, mqtt_handle);
                });
            }
            // Check MQTT.
            Ok(_notification) = eventloop.poll() => {
                // match notification {
                //     rumqttc::Event::Incoming(packet) => { println!("[MQTT] Received: {:?}", packet) },
                //     rumqttc::Event::Outgoing(outgoing) => { println!("[MQTT] Sent: {:?}", outgoing) },
                // }
            }
        }
    }
}

#[derive(Debug, Serialize)]
struct TelemetryMessage {
    ground_station_id: String,
    timestamp: DateTime<Utc>,
    payload: Vec<u8>,
}

impl TelemetryMessage {
    pub fn new(
        ground_station_id: impl Into<String>,
        timestamp: DateTime<Utc>,
        payload: Vec<u8>,
    ) -> Self {
        Self {
            ground_station_id: ground_station_id.into(),
            timestamp,
            payload,
        }
    }
}
