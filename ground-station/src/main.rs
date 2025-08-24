use antenna_controller::{self, AntennaController, mock::MockController};
use demod::{Demodulator, example::ExampleDemod};
use framing::{deframe::Deframer, mock::MockDeframer};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use tracking::{Tracker, get_next_pass};
mod time;

#[cfg(feature = "time_mock")]
use crate::time::MockClock as Clock;
#[cfg(not(feature = "time_mock"))]
use crate::time::SystemClock as Clock;

use crate::time::TimeProvider;

#[tokio::main]
async fn main() {
    #[cfg(feature = "time_mock")]
    println!("Using mock time.");
    #[cfg(not(feature = "time_mock"))]
    println!("Using real system time.");

    let observer = tracking::Observer::new(-34.6, -58.4, 2.5);
    let elements = tracking::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993".as_bytes(),
        "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648".as_bytes(),
    )
    .unwrap();

    let mut next_pass = get_next_pass(
        &observer,
        &elements,
        Clock::now(),
        Duration::from_secs_f64(3600.0 * 6.0),
    )
    .unwrap();

    dbg!(&next_pass);

    let mut timer = Duration::from_secs_f64(next_pass.start - Clock::now().timestamp() as f64);
    println!("\nNext pass is in {:?} seconds.\n", timer.as_secs());

    let sleep = tokio::time::sleep(timer);
    tokio::pin!(sleep);

    loop {
        tokio::select! {
            _ = &mut sleep => {

                println!("\nSTARTING PASS\n");

                // SETUP TRACK
                let stop = Arc::new(AtomicBool::new(false));

                let (tx_samples, rx_samples) = mpsc::channel();

                let demodulator = ExampleDemod::new();
                let deframer = MockDeframer::new();
                let tracker = Tracker::new(&observer, elements.clone()).unwrap();

                let observations = [0u8; 5]
                    .iter()
                    .map(|_| thread::sleep(Duration::from_secs(1)))
                    .map(move |_| {
                        tracker.track(Clock::now()).unwrap()
                    });


                let controller = Arc::new(Mutex::new(
                    MockController
                ));
                // END SETUP TRACK

                // BEGIN TRACKING
                let bits = demodulator.bits(rx_samples.iter());
                let frames = deframer.frames(bits);

                let stop_clone = stop.clone();
                let sdr_handle = thread::spawn(move || {
                    while !stop_clone.load(Ordering::Relaxed) {
                        tx_samples.send(0f64).unwrap();
                        thread::sleep(Duration::from_millis(200));
                    }
                });

                let stop_clone = stop.clone();
                let controller_clone = controller.clone();

                let tracker_handle = thread::spawn(move || {
                    for obs in observations {
                        dbg!(&obs);

                        controller_clone
                            .lock()
                            .unwrap()
                            .send(obs.azimuth, obs.elevation, "sat-name", 1000)
                            .unwrap();
                    }

                    println!("\nPass ended, stopping SDR and tracker.\n");

                    stop_clone.store(true, Ordering::Relaxed);
                });

                for frame in frames {
                    dbg!(&frame);
                }

                tracker_handle.join().unwrap();
                sdr_handle.join().unwrap();

                // END TRACKING

                // Reschedule next pass
                next_pass = get_next_pass(
                    &observer,
                    &elements,
                    Clock::now(),
                    Duration::from_secs_f64(3600.0 * 6.0),
                )
                .unwrap();

                timer = Duration::from_secs_f64(next_pass.start - Clock::now().timestamp() as f64);
                println!("\nNext pass is in {:?} seconds.\n", timer.as_secs());

                sleep.as_mut().reset(tokio::time::Instant::now() + timer);
            }
        }
    }
}
