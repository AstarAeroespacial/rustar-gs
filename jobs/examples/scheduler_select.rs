use chrono::{Duration as ChronoDuration, Utc};
use jobs::{Job, JobScheduler, ScheduledJob};
use std::time::Duration;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::unbounded_channel::<Job>();
    let mut scheduler = JobScheduler::new();

    // Producer: send two jobs, but wait between them
    tokio::spawn({
        let tx = tx.clone();
        async move {
            // First job at +2s
            let job1 = Job {
                timestamp: Utc::now() + ChronoDuration::seconds(2),
            };
            tx.send(job1).unwrap();

            // wait 3 seconds before sending the next job so the first one has time to execute and is not ovewritten
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Second job at +6s
            let job2 = Job {
                timestamp: Utc::now() + ChronoDuration::seconds(1),
            };
            tx.send(job2).unwrap();
        }
    });

    loop {
        tokio::select! {
            // receive jobs
            Some(job) = rx.recv() => {
                println!("Received job for {:?}", job.timestamp);
                scheduler.set_job(ScheduledJob::from_job(job)).unwrap();
            }

            // execute jobs
            job = scheduler.next_job() => {
                println!("Job fired at {:?}", job.timestamp);
            }
        }
    }
}
