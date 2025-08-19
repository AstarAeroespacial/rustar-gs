use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{self, Instant, Interval};

/// An asynchronous stream that yields ticks at a fixed interval between a start and end time.
///
/// This stream is based on `tokio::time::Interval` and stops automatically when the
/// end `Instant` is reached. Missed ticks are skipped, ensuring subsequent ticks
/// continue at the correct schedule.
///
/// # Examples
///
/// Mapping to `DateTime<Utc>` for real-time observations:
///
/// ```no_run
/// use tokio::time::{Instant, Duration};
/// use futures::StreamExt;
/// use chrono::Utc;
///
/// #[tokio::main]
/// async fn main() {
///     let start = Instant::now() + Duration::from_secs(2);
///     let end = start + Duration::from_secs(10);
///     let period = Duration::from_secs(3);
///
///     let stream = FiniteInterval::new(start, end, period);
///
///     // Map each tick to the actual current UTC time
///     let mut utc_stream = stream.map(|_| Utc::now());
///
///     while let Some(now) = utc_stream.next().await {
///         println!("Observation at {}", now);
///     }
/// }
/// ```
#[pin_project]
pub struct FiniteInterval {
    #[pin]
    interval: Interval,
    end: Instant,
}

impl FiniteInterval {
    /// Creates a new `FiniteInterval` stream.
    ///
    /// # Panics
    ///
    /// Panics if `end` is not strictly after `start`.
    pub fn new(start: Instant, end: Instant, period: Duration) -> Self {
        assert!(end > start, "`end` must be strictly after `start`");

        let mut interval = time::interval_at(start, period);

        // Skipping missed ticks to maintain correct schedule.
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        Self { interval, end }
    }
}

impl Stream for FiniteInterval {
    type Item = Instant;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        // Stop the stream when the end instant has been reached.
        if Instant::now() >= *this.end {
            return Poll::Ready(None);
        }

        match this.interval.poll_tick(cx) {
            Poll::Ready(instant) => Poll::Ready(Some(instant)),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use tokio::time::{self as tokio_time, Duration as TokioDuration, Instant as TokioInstant};

    #[tokio::test(start_paused = true)]
    async fn yields_all_ticks() {
        let start = Instant::now() + Duration::from_secs(2);
        let end = start + Duration::from_secs(10);
        let period = Duration::from_secs(3);

        let mut stream = FiniteInterval::new(start, end, period);

        let mut ticks = vec![];

        // Keep advancing by 1s until stream ends
        loop {
            tokio::time::advance(Duration::from_secs(1)).await;
            match stream.next().await {
                Some(tick) => ticks.push(tick),
                None => break,
            }
        }

        // Ensure ticks are strictly increasing
        for w in ticks.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn stops_after_end_instant() {
        let start = TokioInstant::now() + TokioDuration::from_secs(1);
        let end = start + TokioDuration::from_secs(3);
        let period = TokioDuration::from_secs(1);

        let mut stream = FiniteInterval::new(start, end, period);

        // Advance beyond the end instant
        tokio_time::advance(TokioDuration::from_secs(10)).await;

        // Stream should have ended
        assert!(stream.next().await.is_none());
    }

    #[tokio::test(start_paused = true)]
    async fn ticks_are_monotonic() {
        let start = TokioInstant::now() + TokioDuration::from_secs(1);
        let end = start + TokioDuration::from_secs(5);
        let period = TokioDuration::from_secs(1);

        let mut stream = super::FiniteInterval::new(start, end, period);

        let mut last_tick = None;

        loop {
            // Advance time in small increments
            tokio_time::advance(TokioDuration::from_secs(1)).await;

            match stream.next().await {
                Some(tick) => {
                    if let Some(prev) = last_tick {
                        // Each tick must be strictly later than the previous
                        assert!(tick > prev, "Ticks are not monotonic");
                    }
                    last_tick = Some(tick);
                }
                None => break, // Stream has ended
            }
        }

        // Optionally check that we got at least one tick
        assert!(last_tick.is_some(), "No ticks were emitted");
    }
}
