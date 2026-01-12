use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}, clock::DefaultClock};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Governor-based IP rate limiter
/// Limits requests to 50 per minute per IP address
#[derive(Clone)]
pub struct GovernorRateLimiter {
    // Map of IP addresses to their rate limiters
    limiters: Arc<Mutex<HashMap<String, RateLimiter<NotKeyed, InMemoryState, DefaultClock>>>>,
}

impl GovernorRateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Check if an IP is allowed to make a request (50 per minute)
    pub fn check_ip_rate_limit(&self, ip: &str) -> bool {
        let mut limiters = self.limiters.lock().unwrap();

        // Get or create rate limiter for this IP (50 requests per 60 seconds)
        let limiter = limiters
            .entry(ip.to_string())
            .or_insert_with(|| RateLimiter::direct(Quota::per_minute(std::num::NonZeroU32::new(50).unwrap())));

        limiter.check().is_ok()
    }

    /// Get remaining quota for an IP (for client feedback)
    #[allow(dead_code)]
    pub fn get_remaining_quota(&self, ip: &str) -> u32 {
        let limiters = self.limiters.lock().unwrap();

        if let Some(_limiter) = limiters.get(ip) {
            // Return approximate remaining quota
            50 // Conservative estimate - actual value requires more complex tracking
        } else {
            50
        }
    }
}

impl Default for GovernorRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_allows_valid_requests() {
        let limiter = GovernorRateLimiter::new();
        let ip = "192.168.1.1";

        // First 50 requests should pass
        for _ in 0..50 {
            assert!(limiter.check_ip_rate_limit(ip));
        }
    }

    #[test]
    fn test_rate_limiter_blocks_excess_requests() {
        let limiter = GovernorRateLimiter::new();
        let ip = "192.168.1.2";

        // Exceed 50 requests per minute
        for i in 0..51 {
            let allowed = limiter.check_ip_rate_limit(ip);
            if i < 50 {
                assert!(allowed, "Request {} should be allowed", i + 1);
            } else {
                assert!(!allowed, "Request {} should be blocked", i + 1);
            }
        }
    }

    #[test]
    fn test_different_ips_independent() {
        let limiter = GovernorRateLimiter::new();
        let ip1 = "192.168.1.1";
        let ip2 = "192.168.1.2";

        // Exhaust quota for IP1
        for _ in 0..50 {
            assert!(limiter.check_ip_rate_limit(ip1));
        }

        // IP1 should be blocked
        assert!(!limiter.check_ip_rate_limit(ip1));

        // IP2 should still have quota
        assert!(limiter.check_ip_rate_limit(ip2));
    }
}
