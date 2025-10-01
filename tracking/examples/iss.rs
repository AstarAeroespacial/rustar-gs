use std::thread::sleep;
use std::time::Duration;

use antenna_controller::{AntennaController, serial::SerialAntennaController};
use sgp4::chrono;
use tracking::{Observer, Tracker};

fn main() {
    let elements = sgp4::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25186.50618345  .00006730  00000+0  12412-3 0  9992".as_bytes(),
        "2 25544  51.6343 216.2777 0002492 336.9059  23.1817 15.50384048518002".as_bytes(),
    )
    .unwrap();

    let buenos_aires = Observer::new(-34.6, -58.4, 2.5);

    let tracker = Tracker::new(&buenos_aires, elements).unwrap();

    let sender_port = "/dev/ttyUSB0".to_string();
    let baud_rate = 9600;

    let mut controller = SerialAntennaController::new(&sender_port, baud_rate)
        .expect("Failed to open serial port (sender)");

    loop {
        let now = chrono::Utc::now();
        if let Ok(observation) = tracker.track(now) {
            let az = observation.azimuth;
            let el = observation.elevation;

            if let Err(e) = controller.send(az, el, "ISS", 145800) {
                eprintln!("Error sending data: {:?}", e);
            }
        } else {
            eprintln!("No observation available");
        }

        sleep(Duration::from_secs(1));
    }
}
