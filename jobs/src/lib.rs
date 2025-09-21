use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::{
    sync::Notify,
    time::{Duration, Instant},
};

#[derive(Debug, PartialEq)]
pub struct Job {
    pub timestamp: DateTime<Utc>,
}

/// A job scheduled to run at a specific `Instant`.
#[derive(Debug)]
pub struct ScheduledJob {
    /// The time when the job should fire.
    pub instant: Instant,
    /// The job to run.
    pub job: Job,
}

impl ScheduledJob {
    /// Create a `ScheduledJob` from a `Job`, converting its UTC timestamp
    /// into a Tokio `Instant`.
    pub fn from_job(job: Job) -> Self {
        // Convert job.timestamp (DateTime<Utc>) into std::time::Duration
        let now_utc = Utc::now();
        let duration = job
            .timestamp
            .signed_duration_since(now_utc)
            .to_std()
            .unwrap_or(Duration::from_secs(0)); // if it's in the past, clamp to now

        let instant = Instant::now() + duration;
        ScheduledJob { instant, job }
    }
}

/// Schedules and executes jobs at specified instants.
///
/// Only one job can be scheduled at a time. When a job is scheduled,
/// [`next_job`](JobScheduler::next_job) will wait until its instant and then return it.
/// If no job is set, [`next_job`](JobScheduler::next_job) will wait until one is scheduled.
///
/// # Example
/// ```
/// use chrono::{Utc, Duration as ChronoDuration};
/// use jobs::{JobScheduler, ScheduledJob, Job};
///
/// #[tokio::main]
/// async fn main() {
///     let mut scheduler = JobScheduler::new();
///
///     // Create a job 1 second from now
///     let job = Job { timestamp: Utc::now() + ChronoDuration::seconds(1) };
///     scheduler.set_job(ScheduledJob::from_job(job)).unwrap();
///
///     // Wait for the job to fire
///     let job = scheduler.next_job().await.unwrap();
///     assert!(job.timestamp <= Utc::now());
/// }
/// ```
pub struct JobScheduler {
    current: Option<ScheduledJob>,
    notify: Arc<Notify>,
}

impl JobScheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            current: None,
            notify: Arc::new(Notify::new()),
        }
    }

    /// Schedule a job to run at a specific `Instant`.
    ///
    /// Replaces any previously scheduled job.
    ///
    /// # Errors
    /// Returns [`JobSchedulerError::JobInPast`] if the job is scheduled
    /// for an instant earlier than `Instant::now()`.
    pub fn set_job(&mut self, job: ScheduledJob) -> Result<(), JobSchedulerError> {
        let now = Instant::now();
        if job.instant < now {
            return Err(JobSchedulerError::JobInPast);
        }

        self.current = Some(job);
        self.notify.notify_one(); // wake any waiter
        Ok(())
    }

    /// Wait for the next scheduled job.
    ///
    /// If a job is already scheduled, waits until its instant and returns it.
    /// If no job is scheduled, this call will suspend until one is set.
    pub async fn next_job(&mut self) -> Option<Job> {
        loop {
            if let Some(job) = self.current.take() {
                tokio::time::sleep_until(job.instant).await;
                return Some(job.job);
            }

            // wait until a job is scheduled
            self.notify.notified().await;
        }
    }
}

/// Error returned when scheduling a job fails.
#[derive(Debug)]
pub enum JobSchedulerError {
    /// The job was scheduled in the past.
    JobInPast,
}
