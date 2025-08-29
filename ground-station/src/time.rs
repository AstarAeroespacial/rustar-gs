use chrono::{DateTime, Utc};

pub trait TimeProvider {
    fn now() -> DateTime<Utc>;
}

pub struct SystemClock;

impl TimeProvider for SystemClock {
    fn now() -> DateTime<Utc> {
        Utc::now()
    }
}

pub struct MockClock;

impl TimeProvider for MockClock {
    fn now() -> DateTime<Utc> {
        // This timestamp is just before an ISS pass.
        DateTime::from_timestamp(1756065315, 0).unwrap()
    }
}
