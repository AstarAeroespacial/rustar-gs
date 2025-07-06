use predict_rs::{consts::DEG_TO_RAD, predict::PredictObserver};
use sgp4::chrono;
use tracking::{self, Tracker};

fn main() {
    let elements = sgp4::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25186.50618345  .00006730  00000+0  12412-3 0  9992".as_bytes(),
        "2 25544  51.6343 216.2777 0002492 336.9059  23.1817 15.50384048518002".as_bytes(),
    )
    .unwrap();

    let buenos_aires = PredictObserver {
        name: "Buenos Aires".to_string(),
        latitude: -34.6 * DEG_TO_RAD,
        longitude: -58.4 * DEG_TO_RAD,
        altitude: 2.5,
        min_elevation: 0.0,
    };

    let tracker = Tracker::new(&buenos_aires, &elements).unwrap();

    let now = chrono::Utc::now().timestamp();

    let observation = tracker.track(now).unwrap();

    println!("{:?}", observation);
}
