use chrono::{Duration, Utc};
use jobs::{Job, JobScheduler, ScheduledJob};

#[tokio::main]
async fn main() {
    let mut scheduler = JobScheduler::new();

    // create a Job with a UTC timestamp 2 seconds in the future
    let job = Job {
        timestamp: Utc::now() + Duration::seconds(2),
    };

    // convert it to ScheduledJob automatically
    scheduler.set_job(ScheduledJob::from_job(job)).unwrap();

    println!("Waiting for job...");
    if let Some(job) = scheduler.next_job().await {
        println!("Job fired at {:?}", job.timestamp);
    }
}
