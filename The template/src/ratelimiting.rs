use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct RateLimiter {
    requests: HashMap<String, Vec<Instant>>, // Store timestamps of requests by IP
    max_requests: usize,
    window_duration: Duration,
}

impl RateLimiter {
    // Constructor to create a new RateLimiter with max requests and window duration
    pub fn new(max_requests: usize, window_duration: Duration) -> Self {
        RateLimiter {
            requests: HashMap::new(),
            max_requests,
            window_duration,
        }
    }

    // Method to check if a request is allowed for a specific IP
    pub fn allow_request(&mut self, ip: &str) -> bool {
        let now = Instant::now(); // Get the current time
        let entry = self.requests.entry(ip.to_string()).or_insert_with(Vec::new);

        // Remove requests outside the time window
        entry.retain(|&timestamp| now.duration_since(timestamp) < self.window_duration);

        // Check if the IP has exceeded the request limit
        if entry.len() < self.max_requests {
            // Add the current request timestamp
            entry.push(now);
            true
        } else {
            false
        }
    }
}
