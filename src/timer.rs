use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Counts down from a starting duration. Created once, polled each frame.
pub struct Countdown {
    start: Instant,
    total: Duration,
}

impl Countdown {
    pub fn new(total_secs: u64) -> Self {
        Self {
            start: Instant::now(),
            total: Duration::from_secs(total_secs.max(1)),
        }
    }

    pub fn remaining(&self) -> Duration {
        self.total.checked_sub(self.start.elapsed()).unwrap_or(Duration::ZERO)
    }

    pub fn remaining_secs(&self) -> u64 {
        self.remaining().as_secs()
    }

    pub fn expired(&self) -> bool {
        self.start.elapsed() >= self.total
    }
}

/// Counts up from creation. Used for the session duration display.
pub struct Elapsed {
    start: Instant,
}

impl Elapsed {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    pub fn format(&self) -> String {
        let s = self.start.elapsed().as_secs();
        let h = s / 3600;
        let m = (s / 60) % 60;
        let s = s % 60;
        if h > 0 {
            format!("{:02}:{:02}:{:02}", h, m, s)
        } else {
            format!("{:02}:{:02}", m, s)
        }
    }
}

/// Current time in milliseconds since the Unix epoch. Returns 0 if the system
/// clock is set before 1970, which we don't bother guarding against beyond that.
pub fn unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Format a Unix-ms timestamp as "HH:MM" in UTC. We avoid pulling in chrono /
/// time to keep the dependency surface minimal — UTC is consistent across
/// platforms and is good enough for short-session chat timestamps.
pub fn format_clock(unix_ms: u64) -> String {
    let secs_today = (unix_ms / 1000) % 86_400;
    let h = secs_today / 3600;
    let m = (secs_today / 60) % 60;
    format!("{:02}:{:02}", h, m)
}
