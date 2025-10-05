// Simple rate limiter for throttling expensive operations

use std::sync::Mutex;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    pub fn new(min_interval_ms: u64) -> Self {
        Self {
            // Initialize with a time far in the past to allow first request
            last_request: Mutex::new(Instant::now() - Duration::from_secs(100)),
            min_interval: Duration::from_millis(min_interval_ms),
        }
    }

    /// Check if enough time has passed since last request and update the timestamp
    /// Returns Ok(()) if request is allowed, Err with message if rate limited
    pub fn check_and_update(&self) -> Result<(), String> {
        let mut last = self.last_request.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last) < self.min_interval {
            let remaining = self.min_interval - now.duration_since(*last);
            return Err(format!(
                "Rate limit exceeded. Please wait {:.1} seconds before trying again.",
                remaining.as_secs_f32()
            ));
        }

        *last = now;
        Ok(())
    }

    /// Check rate limit without updating (for read-only checks)
    pub fn check(&self) -> Result<(), String> {
        let last = self.last_request.lock().unwrap();
        let now = Instant::now();

        if now.duration_since(*last) < self.min_interval {
            let remaining = self.min_interval - now.duration_since(*last);
            return Err(format!(
                "Rate limit exceeded. Please wait {:.1} seconds.",
                remaining.as_secs_f32()
            ));
        }

        Ok(())
    }

    /// Reset the rate limiter (for testing purposes)
    /// Note: Public to allow integration tests to reset state
    pub fn reset(&self) {
        let mut last = self.last_request.lock().unwrap();
        *last = Instant::now() - Duration::from_secs(100);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_rate_limiter_allows_first_request() {
        let limiter = RateLimiter::new(1000); // 1 second
        assert!(limiter.check_and_update().is_ok());
    }

    #[test]
    fn test_rate_limiter_blocks_rapid_requests() {
        let limiter = RateLimiter::new(100); // 100ms

        // First request should succeed
        assert!(limiter.check_and_update().is_ok());

        // Immediate second request should fail
        assert!(limiter.check_and_update().is_err());
    }

    #[test]
    fn test_rate_limiter_allows_after_interval() {
        let limiter = RateLimiter::new(50); // 50ms

        // First request
        assert!(limiter.check_and_update().is_ok());

        // Wait for interval
        sleep(Duration::from_millis(60));

        // Second request after interval should succeed
        assert!(limiter.check_and_update().is_ok());
    }

    #[test]
    fn test_check_does_not_update_timestamp() {
        let limiter = RateLimiter::new(100); // 100ms

        // Update timestamp
        assert!(limiter.check_and_update().is_ok());

        // Check immediately (should fail)
        assert!(limiter.check().is_err());

        // Check again (should still fail, timestamp unchanged)
        assert!(limiter.check().is_err());
    }
}
