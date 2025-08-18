use antenna_controller::{self, AntennaController, SerialAntennaController};
use hdlc::deframer::{Deframer, HdlcDeframer};
use modem::{
    self,
    demodulator::{AfskGnuRadioDemodulator, Demodulator},
};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
// use tokio_stream::Stream;
use tracking::{self, Observation, Observer, Tracker};

type Sample = [u8; 8];

fn main() {}

fn track(elements: tracking::Elements, ground_station: Observer, pass_end: f64) {
    // BEGIN SETUP
    let stop = Arc::new(AtomicBool::new(false));

    // 1. Create tracker.
    let tracker = Tracker::new(&ground_station, elements).unwrap();

    // 2. Init antenna controller.
    let port = "/dev/pts/2".to_string();
    let mut controller = SerialAntennaController::new(&port, 9600).unwrap();

    // Set up the channels to communicate the actors.
    // SDR tx_samples=========rx_samples DEMODULATOR tx_bits======rx_bits DEFRAMER tx_packets====rx_packets
    let (tx_samples, rx_samples) = mpsc::channel::<Vec<Sample>>();
    let (tx_bits, rx_bits) = mpsc::channel();
    let (tx_packets, rx_packets) = mpsc::channel();

    // 3. Init SDR.
    // soapy?

    // 4. Init demod.
    let demodulator = AfskGnuRadioDemodulator::build(
        rx_samples,
        tx_bits,
        PathBuf::from("afks_demod"), // el path donde está el flowgraph a usar
        None::<PathBuf>,
    )
    .unwrap();

    // END SETUP

    // send samples to tx_samples
    let stop_clone = stop.clone();
    let sdr_handler = thread::spawn(move || {
        while !stop_clone.load(Ordering::Relaxed) {
            tx_samples.send(vec![[0u8; 8]]).unwrap();
        }
        drop(tx_samples);
    });

    // Run the tracker.
    let stop_clone = stop.clone();
    let tracker_handler = thread::spawn(move || {
        loop {
            let now = chrono::Utc::now();

            if now.timestamp() as f64 > pass_end {
                // pass has ended
                stop_clone.store(true, Ordering::Relaxed);
                break;
            }

            let obs = tracker.track(now).unwrap();

            controller
                .send(obs.azimuth, obs.elevation, "sat-name", 1000)
                .unwrap();

            thread::sleep(Duration::from_millis(1000));
        }
    });

    // Run the demodulator.
    thread::spawn(move || {
        demodulator.run();
    });

    // Run the deframer.
    thread::spawn(move || {
        let mut deframer = HdlcDeframer::new(rx_bits, tx_packets);
        deframer.run();
    });

    // Sender.
    thread::spawn(move || {
        while let Ok(packet) = rx_packets.recv() {
            // send mqtt
            dbg!(&packet);
        }
    });

    tracker_handler.join().unwrap();
    sdr_handler.join().unwrap();
    // demod_handler.join().unwrap();
    // deframer_handler.join().unwrap();
    // sender_handler.join().unwrap();
}
