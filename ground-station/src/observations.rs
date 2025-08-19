use chrono::{DateTime, Utc};
use futures::Stream;
use pin_project::pin_project;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::time::{self, Instant, Interval};

// Ideas:
// Build:
// with frequency
// with period
// with total
// with start and length
// with end and length

#[pin_project]
pub struct Observations {
    #[pin]
    interval: Interval,
    end: DateTime<Utc>,
}

impl Observations {
    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>, period: Duration) -> Self {
        assert!(end > start, "`end` must be strictly after `start`");

        let now = Utc::now();

        let delay_until_start = (start - now)
            .to_std()
            .expect("`start` must be in the future");

        let start_instant = Instant::now() + delay_until_start;

        let mut interval = time::interval_at(start_instant, period);
        // Even if we skip ticks for some reason, we skip them, because we want the subsequent
        // observations to be yielded at the correct time. Otherwise the antenna would be
        // "catching up" (`Burst`) or "late" (`Delay`).
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        Self { interval, end }
    }
}

impl Stream for Observations {
    type Item = DateTime<Utc>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let now = Utc::now();

        // Stop if we've reached the end wall-clock time.
        if now >= *this.end {
            return Poll::Ready(None);
        }

        match this.interval.poll_tick(cx) {
            Poll::Ready(_) => Poll::Ready(Some(now)),
            Poll::Pending => Poll::Pending,
        }
    }
}
