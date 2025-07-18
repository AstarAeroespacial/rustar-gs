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

    let tracker = Tracker::new(&buenos_aires, &elements).unwrap();

    let now = chrono::Utc::now().timestamp();

    let observation = tracker.track(now).unwrap();

    println!("{:?}", observation);
}
