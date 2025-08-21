use antenna_controller::{self, AntennaController, SerialAntennaController};
use chrono::Utc;
use framing::deframe::Deframer;
use framing::hdlc::HdlcDeframer;
use modem::{Demodulator, afsk1200::AfskDemodulator};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tracking::{Pass, Tracker, get_next_pass};

#[tokio::main]
async fn main() {
    let observer = tracking::Observer::new(-34.6, -58.4, 2.5);
    let elements = tracking::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25186.50618345  .00006730  00000+0  12412-3 0  9992".as_bytes(),
        "2 25544  51.6343 216.2777 0002492 336.9059  23.1817 15.50384048518002".as_bytes(),
    )
    .unwrap();

    let mut next_pass = get_next_pass(
        &observer,
        &elements,
        Utc::now(),
        Duration::from_secs_f64(3600.0 * 6.0),
    )
    .unwrap();

    let mut timer = get_duration_until_pass(next_pass);

    let sleep = tokio::time::sleep(timer);
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            _ = &mut sleep => {

                // SETUP TRACK
                let stop = Arc::new(AtomicBool::new(false));

                let (tx_samples, rx_samples) = mpsc::channel();

                let demodulator = AfskDemodulator::new();
                let deframer = HdlcDeframer::new();

                let tracker = Tracker::new(&observer, elements.clone()).unwrap();

                let controller = Arc::new(Mutex::new(
                    SerialAntennaController::new("/dev/pts/2", 9600).unwrap(),
                ));
                // END SETUP TRACK

                // BEGIN TRACKING
                let bits = demodulator.bits(rx_samples.iter());
                let frames = deframer.frames(bits);

                let stop_clone = stop.clone();
                let sdr_handle = thread::spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        tx_samples.send(0f64).unwrap();
                    }
                });

                let stop_clone = stop.clone();
                let pass_end = next_pass.end;
                let controller_clone = controller.clone();

                let tracker_handle = thread::spawn(move || {
                    loop {
                        let now = chrono::Utc::now();

                        if now.timestamp() as f64 > pass_end {
                            // pass has ended
                            stop_clone.store(true, Ordering::Relaxed);
                            break;
                        }

                        let obs = tracker.track(now).unwrap();

                        controller_clone
                            .lock()
                            .unwrap()
                            .send(obs.azimuth, obs.elevation, "sat-name", 1000)
                            .unwrap();

                        thread::sleep(Duration::from_millis(1000));
                    }
                });

                for frame in frames {
                    dbg!(&frame);
                }

                tracker_handle.join().unwrap();
                sdr_handle.join().unwrap();

                // END TRACKING

                next_pass = get_next_pass(
                    &observer,
                    &elements,
                    Utc::now(),
                    Duration::from_secs_f64(3600.0 * 6.0),
                )
                .unwrap();

                timer = get_duration_until_pass(next_pass);

                sleep.as_mut().reset(tokio::time::Instant::now() + timer);
            }
        }
    }
}

fn get_duration_until_pass(pass: Pass) -> Duration {
    let now = chrono::Utc::now();
    let now_timestamp = now.timestamp() as f64;

    Duration::from_secs_f64(pass.start - now_timestamp)
}
