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
    ///
    /// This method is thread-safe and updates the internal timestamp on success.
    ///
    /// # Returns
    /// - `Ok(())` if the request is allowed (enough time has passed)
    /// - `Err(f64)` with remaining seconds to wait if rate limited
    ///
    /// # Examples
    /// ```no_run
    /// use budget_balancer_lib::utils::rate_limiter::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(2000); // 2 second minimum interval
    /// match limiter.check_and_update() {
    ///     Ok(()) => println!("Request allowed"),
    ///     Err(secs) => println!("Rate limited, wait {:.1}s", secs),
    /// }
    /// ```
    pub fn check_and_update(&self) -> Result<(), f64> {
        let mut last = match self.last_request.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Rate limiter mutex was poisoned, recovering");
                poisoned.into_inner()
            }
        };
        let now = Instant::now();

        if now.duration_since(*last) < self.min_interval {
            let remaining = self.min_interval - now.duration_since(*last);
            return Err(remaining.as_secs_f64());
        }

        *last = now;
        Ok(())
    }

    /// Check rate limit without updating the timestamp (read-only)
    ///
    /// This method checks if a request would be allowed without modifying state.
    /// Useful for preview/validation without consuming the rate limit.
    ///
    /// # Returns
    /// - `Ok(())` if a request would be allowed
    /// - `Err(f64)` with remaining seconds to wait if currently rate limited
    ///
    /// # Examples
    /// ```no_run
    /// use budget_balancer_lib::utils::rate_limiter::RateLimiter;
    ///
    /// let limiter = RateLimiter::new(2000);
    /// if limiter.check().is_ok() {
    ///     // Safe to proceed, can call check_and_update()
    /// }
    /// ```
    pub fn check(&self) -> Result<(), f64> {
        let last = match self.last_request.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Rate limiter mutex was poisoned during check, recovering");
                poisoned.into_inner()
            }
        };
        let now = Instant::now();

        if now.duration_since(*last) < self.min_interval {
            let remaining = self.min_interval - now.duration_since(*last);
            return Err(remaining.as_secs_f64());
        }

        Ok(())
    }

    /// Reset the rate limiter to allow immediate requests
    ///
    /// This resets the internal timestamp to allow the next request immediately.
    /// Primarily intended for testing, but safe to use in production if needed.
    ///
    /// # Note
    /// This method is public to allow integration tests to reset state between test runs.
    pub fn reset(&self) {
        let mut last = match self.last_request.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                tracing::warn!("Rate limiter mutex was poisoned during reset, recovering");
                poisoned.into_inner()
            }
        };
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
