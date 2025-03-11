use std::time::{Duration, Instant};
use std::collections::HashMap;

pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>,
    warning_threshold: usize,
    ban_threshold: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(warning_threshold: usize, ban_threshold: usize, window_duration: Duration) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            warning_threshold,
            ban_threshold,
            window_duration,
        }
    }

    pub fn check_request(&mut self, ip: &str) -> RateLimitStatus {
        let now = Instant::now();
        let entry = self.requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove old requests outside the time window
        entry.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        if entry.len() >= self.ban_threshold {
            RateLimitStatus::Banned
        } else if entry.len() >= self.warning_threshold {
            entry.push(now);
            RateLimitStatus::Warning
        } else {
            entry.push(now);
            RateLimitStatus::Allowed
        }
    }
}

pub enum RateLimitStatus {
    Allowed,
    Warning,
    Banned,
}

