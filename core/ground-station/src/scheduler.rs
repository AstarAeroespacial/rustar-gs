use std::sync::Arc;
use tokio::{sync::Notify, time::Instant};

/// A task scheduled to run at a specific `Instant`.
#[derive(Debug)]
pub struct Task<T> {
    /// The time when the task should fire.
    pub instant: Instant,
    /// The task data to run.
    pub data: T,
}

impl<T> Task<T> {
    /// Create a new `Task` with the given instant and data.
    pub fn new(instant: Instant, data: T) -> Self {
        Task { instant, data }
    }
}

/// Schedules and executes tasks at specified instants.
///
/// Only one task can be scheduled at a time. When a task is scheduled,
/// [`next`](Scheduler::next) will wait until its instant and then return it.
/// If no task is set, [`next`](Scheduler::next) will wait until one is scheduled.
///
/// # Example
///
/// ```no_run
/// use ground_station::scheduler::{Scheduler, Task};
/// use tokio::time::{Instant, Duration};
///
/// #[tokio::main]
/// async fn main() {
///     let mut scheduler = Scheduler::new();
///
///     // Schedule a task to run 1 second from now
///     let task = Task::new(Instant::now() + Duration::from_secs(1), "my_task");
///     scheduler.schedule(task).unwrap();
///
///     // Wait for the task to fire
///     let task_data = scheduler.next().await;
///     assert_eq!(task_data, "my_task");
/// }
/// ```
pub struct Scheduler<T> {
    current: Option<Task<T>>,
    notify: Arc<Notify>,
}

impl<T> Scheduler<T> {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            current: None,
            notify: Arc::new(Notify::new()),
        }
    }

    /// Schedule a task to run at a specific `Instant`.
    ///
    /// Replaces any previously scheduled task.
    ///
    /// # Errors
    /// Returns [`SchedulerError::TaskInPast`] if the task is scheduled
    /// for an instant earlier than `Instant::now()`.
    ///
    /// # Example
    ///
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     use ground_station::scheduler::{Scheduler, Task, SchedulerError};
    ///     use tokio::time::{Instant, Duration};
    ///
    ///     let mut scheduler = Scheduler::new();
    ///     let task = Task::new(Instant::now() + Duration::from_secs(5), "data");
    ///
    ///     // Schedule the task
    ///     scheduler.schedule(task).unwrap();
    ///
    ///     dbg!("Waiting for job...");
    ///
    ///     let task = scheduler.next().await;
    ///     dbg!(&task);
    /// }
    /// ```
    pub fn schedule(&mut self, task: impl Into<Task<T>>) -> Result<(), SchedulerError> {
        let now = Instant::now();
        let task = task.into();

        if task.instant < now {
            println!("[SCHEDULER] Task is in the past!");
            return Err(SchedulerError::TaskInPast);
        }

        println!(
            "[SCHEDULER] Scheduling task for {:?} from now",
            task.instant.saturating_duration_since(now)
        );
        self.current = Some(task);
        self.notify.notify_one(); // wake any waiter
        println!("[SCHEDULER] Notification sent");

        Ok(())
    }

    /// Wait for the next scheduled task.
    ///
    /// If a task is already scheduled, waits until its instant and returns it.
    /// If no task is scheduled, this call will suspend until one is set.
    ///
    /// This method is cancel-safe: if the future is dropped before completing,
    /// the scheduled task remains in the scheduler and will be returned by
    /// the next call to `next()`.
    pub async fn next(&mut self) -> T {
        loop {
            // Create the notified future BEFORE checking current
            // This ensures we don't miss notifications
            let notified = self.notify.notified();

            // Check if there's a task WITHOUT taking it (cancel-safety!)
            if let Some(ref task) = self.current {
                let now = Instant::now();
                let wait_duration = task.instant.saturating_duration_since(now);
                println!("[SCHEDULER] Task found, waiting {:?}", wait_duration);

                // Sleep until the task's instant
                // If this future is cancelled, we haven't mutated state yet!
                tokio::time::sleep_until(task.instant).await;

                // NOW take the task after sleep completes
                // This is the only mutation, and it happens atomically
                println!("[SCHEDULER] Task firing now!");
                return self.current.take().unwrap().data;
            }

            println!("[SCHEDULER] No task scheduled, waiting for notification...");
            // wait until a task is scheduled
            notified.await;
            println!("[SCHEDULER] Notification received, checking for task...");
        }
    }
}

/// Error returned when scheduling a task fails.
#[derive(Debug)]
pub enum SchedulerError {
    /// The task was scheduled in the past.
    TaskInPast,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Duration;

    #[tokio::test]
    async fn test_basic_scheduler() {
        let mut scheduler = Scheduler::new();

        // Schedule a task to run 100ms from now
        let task = Task::new(Instant::now() + Duration::from_millis(100), "test_task");
        scheduler.schedule(task).unwrap();

        // Wait for the task to fire
        let task_data = scheduler.next().await;
        assert_eq!(task_data, "test_task");
    }

    #[tokio::test]
    async fn test_scheduler_with_select() {
        use tokio::sync::mpsc;

        let (tx, mut rx) = mpsc::unbounded_channel::<&str>();
        let mut scheduler = Scheduler::new();

        // Producer: send two tasks, but wait between them
        tokio::spawn(async move {
            // First task at +100ms
            tx.send("first_task").unwrap();

            // Wait 150ms before sending the next task so the first one has time to execute
            tokio::time::sleep(Duration::from_millis(150)).await;

            // Second task at +50ms from now (so +200ms from start)
            tx.send("second_task").unwrap();
        });

        let mut executed_tasks = Vec::new();

        // Run for a limited time to avoid infinite loop in test
        let timeout = tokio::time::sleep(Duration::from_millis(500));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                // Receive tasks from channel and schedule them
                Some(task_name) = rx.recv() => {
                    let task = Task::new(
                        Instant::now() + Duration::from_millis(50),
                        task_name
                    );
                    scheduler.schedule(task).unwrap();
                }

                // Execute scheduled tasks
                task = scheduler.next() => {
                    executed_tasks.push(task);

                    // Stop after executing 2 tasks
                    if executed_tasks.len() == 2 {
                        break;
                    }
                }

                // Timeout to prevent test from hanging
                _ = &mut timeout => {
                    panic!("Test timed out");
                }
            }
        }

        assert_eq!(executed_tasks.len(), 2);
        assert_eq!(executed_tasks[0], "first_task");
        assert_eq!(executed_tasks[1], "second_task");
    }

    #[tokio::test]
    async fn test_schedule_task_in_past_returns_error() {
        let mut scheduler = Scheduler::new();

        // Try to schedule a task in the past
        let past_task = Task::new(Instant::now() - Duration::from_secs(1), "past_task");

        let result = scheduler.schedule(past_task);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_scheduler_replaces_previous_task() {
        let mut scheduler = Scheduler::new();

        // Schedule first task
        let task1 = Task::new(Instant::now() + Duration::from_millis(200), "task1");
        scheduler.schedule(task1).unwrap();

        // Immediately schedule second task (should replace first)
        let task2 = Task::new(Instant::now() + Duration::from_millis(100), "task2");
        scheduler.schedule(task2).unwrap();

        // Should execute task2, not task1
        let task_data = scheduler.next().await;
        assert_eq!(task_data, "task2");
    }
}
