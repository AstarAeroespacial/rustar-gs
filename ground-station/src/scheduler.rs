use std::{collections::BinaryHeap, pin::pin, task::Poll};

use futures::Stream;
use pin_project::pin_project;
use tokio::time::{Instant, Sleep};

struct Pass {}

enum Event {
    Pass(Pass),
    Retry,
}

struct ScheduledEvent {
    // priority: PriorityLevel,
    time: Instant,
    event: Event,
}

impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse so BinaryHeap pops the earliest event first
        other.time.cmp(&self.time)
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, o: &Self) -> bool {
        self.time == o.time
    }
}

impl Eq for ScheduledEvent {}

#[pin_project]
struct Scheduler {
    queue: BinaryHeap<ScheduledEvent>,
    #[pin]
    sleeper: Option<Sleep>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            queue: BinaryHeap::new(),
            sleeper: None,
        }
    }

    fn push(&mut self, event: ScheduledEvent) {
        self.queue.push(event);
    }
}

impl Stream for Scheduler {
    type Item = Event;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let mut this = self.project();

        // If we have an event and it's due, pop and return it
        if let Some(next) = this.queue.peek() {
            if Instant::now() >= next.time {
                return Poll::Ready(Some(this.queue.pop().unwrap().event));
            }

            // Otherwise, set up the sleeper to wake us at the right time
            if this.sleeper.is_none() {
                this.sleeper.set(Some(tokio::time::sleep_until(next.time)));
            }
        }

        // Poll the sleeper if it exists
        if let Some(sleep) = this.sleeper.as_mut().as_pin_mut() {
            match sleep.poll(cx) {
                Poll::Ready(()) => {
                    // Time has arrived, clear sleeper and wake on next poll
                    this.sleeper.set(None);
                    cx.waker().wake_by_ref(); // force re-poll so we pop the event

                    Poll::Pending
                }
                Poll::Pending => Poll::Pending,
            }
        } else {
            // No events in the queue
            Poll::Pending
        }
    }
}
