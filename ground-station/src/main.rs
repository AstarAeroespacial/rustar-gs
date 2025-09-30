use crate::config::Config;
use crate::time::TimeProvider;
use antenna_controller::{self, AntennaController, mock::MockController};
use api::{ApiDoc, add_job, root};
use axum::{
    Router,
    routing::{get, post},
};
use demod::gr_mock::GrBitSource;
use framing::{deframer::Deframer, hdlc_deframer::HdlcDeframer};
use mqtt_client::{receiver::MqttReceiver, sender::MqttSender};
use packetizer::{Packetizer, packetizer::TelemetryRecordPacketizer};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    net::TcpListener,
    sync::mpsc,
    task::spawn_blocking,
};
use tokio_stream::{self, StreamExt};
use tracking::{Tracker, get_next_pass};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "time_mock")]
use crate::time::MockClock as Clock;
#[cfg(not(feature = "time_mock"))]
use crate::time::SystemClock as Clock;

mod config;
mod time;

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

    #[cfg(feature = "time_mock")]
    println!("Using mock time.");
    #[cfg(not(feature = "time_mock"))]
    println!("Using real system time.");

    let mut observer = tracking::Observer::new(
        config.ground_station.latitude,
        config.ground_station.longitude,
        config.ground_station.altitude,
    );
    let mut elements = tracking::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993".as_bytes(),
        "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648".as_bytes(),
    )
    .unwrap();

    let (mqtt_send, eventloop) =
        MqttSender::new(&config.mqtt.host, config.mqtt.port, config.mqtt.timeout());
    let mut mqtt_recv = MqttReceiver::from_client(mqtt_send.client(), eventloop);
    mqtt_recv.subscribe("sat1/control").await.unwrap();

    let mut next_pass = get_next_pass(
        &observer,
        &elements,
        Clock::now(),
        Duration::from_secs_f64(3600.0 * 6.0),
    )
    .unwrap();

    let mut timer =
        Duration::from_secs_f64((next_pass.start - Clock::now().timestamp() as f64).max(1.0));

    println!("\nNext pass is in {:?} seconds.\n", timer.as_secs());

    let sleep = tokio::time::sleep(timer);
    tokio::pin!(sleep);

    // Estado para controlar si ya hay un tracking en progreso
    let mut tracking_in_progress = false;

    // Canal para comunicar el siguiente pase
    let (next_pass_tx, mut next_pass_rx) = mpsc::channel(1);

    let addr = "localhost:9999";
    let listener = TcpListener::bind(&addr).await.unwrap();

    let router = Router::new()
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/", get(root))
        .route("/jobs", post(add_job));

    tokio::spawn(async move {
        println!("Swagger UI available at http://{addr}/docs");

        axum::serve(listener, router).await.unwrap();
    });

    loop {
        tokio::select! {
            // Cuando llega un nuevo pase calculado, actualizar timer
            Some(new_pass) = next_pass_rx.recv() => {
                next_pass = new_pass;
                tracking_in_progress = false; // Marcar que el tracking terminó

                let time_until_pass = (next_pass.start - Clock::now().timestamp() as f64).max(1.0);
                timer = Duration::from_secs_f64(time_until_pass);

                println!("\nNext pass calculated! Will start in {:?} seconds.\n", timer.as_secs());

                sleep.as_mut().reset(tokio::time::Instant::now() + timer);
            }

            _ = &mut sleep, if !tracking_in_progress => {
                println!("\nSTARTING PASS\n");
                tracking_in_progress = true; // Marcar que comenzó el tracking

                let observer_clone = observer.clone();
                let elements_clone = elements.clone();
                let next_pass_tx_clone = next_pass_tx.clone();

                // Lanzar tracking en background
                tokio::spawn(async move {

                    // INIT SETUP
                    let tracker = Tracker::new(&observer_clone, elements_clone.clone()).unwrap();
                    let stop = Arc::new(AtomicBool::new(false));

                    let deframer = HdlcDeframer::new();
                    let packetizer = TelemetryRecordPacketizer::new();
                    let controller = Arc::new(Mutex::new(MockController));
                    // END SETUP

                    // TRACKING
                    let stop_clone = stop.clone();
                    let controller_clone = controller.clone();
                    let tracker_handle = tokio::spawn(async move {
                        for i in 0..5 {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            let obs = tracker.track(Clock::now()).unwrap();

                            println!("Tracking step {}: Az={:.1}°, El={:.1}°",
                                     i, obs.azimuth.to_degrees(), obs.elevation.to_degrees());

                            controller_clone
                                .lock()
                                .unwrap()
                                .send(obs.azimuth.to_degrees(), obs.elevation.to_degrees(), "ISS", 145800)
                                .unwrap();
                        }

                        println!("\nPass ended, stopping SDR and tracker.\n");
                        stop_clone.store(true, Ordering::Relaxed);
                    });

                    // SAMPLES
                    let stop_clone = stop.clone();
                    let frame_handle = tokio::spawn(async move {
                        let bits = GrBitSource::new();
                        let frames = deframer.frames(bits);
                        let mut packets = packetizer.packets(frames);

                        while !stop_clone.load(Ordering::Relaxed) {
                            if let Some(packet) = packets.next() {
                                // TODO: send via MQTT here.
                                dbg!(&packet);
                            }
                        }
                    });

                    let _ = tokio::join!(tracker_handle, /*sdr_handle,*/ frame_handle);

                    // NEXT PASS CALCULATION
                    println!("Calculating next pass...");
                    let observer_for_calc = observer_clone.clone();
                    let elements_for_calc = elements_clone.clone();

                    let new_pass = spawn_blocking(move || {
                        get_next_pass(
                            &observer_for_calc,
                            &elements_for_calc,
                            Clock::now(),
                            Duration::from_secs_f64(3600.0 * 6.0)
                        ).unwrap()
                    }).await.unwrap();

                    next_pass_tx_clone.send(new_pass).await.unwrap();
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
