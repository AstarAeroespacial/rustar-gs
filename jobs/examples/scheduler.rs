use chrono::{Duration, Utc};
use jobs::{Job, JobScheduler, ScheduledJob};

#[tokio::main]
async fn main() {
    let mut scheduler = JobScheduler::new();

    let elements = tracking::Elements::from_tle(
        Some("ISS (ZARYA)".to_owned()),
        "1 25544U 98067A   25235.75642456  .00011222  00000+0  20339-3 0  9993".as_bytes(),
        "2 25544  51.6355 332.1708 0003307 260.2831  99.7785 15.50129787525648".as_bytes(),
    )
    .unwrap();

    // create a Job with a UTC timestamp 2 seconds in the future
    let job = Job {
        timestamp: Utc::now() + Duration::seconds(2),
        elements,
        satellite_name: "ISS (ZARYA)".to_string(),
    };

    // convert it to ScheduledJob automatically
    scheduler.set_job(ScheduledJob::from_job(job)).unwrap();

    println!("Waiting for job...");

    let job = scheduler.next_job().await;
    println!("Job fired at {:?}", job.timestamp);
}
