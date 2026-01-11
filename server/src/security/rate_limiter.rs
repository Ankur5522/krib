use crate::redis_client::RedisClient;
use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limiter using sliding window algorithm with Redis
#[derive(Clone)]
pub struct RateLimiter {
    redis: RedisClient,
}

#[derive(Debug, Clone, Copy)]
pub enum RateLimitType {
    /// 1 post per 60 seconds
    PostMessage,
    /// 5 reveals per hour
    ContactReveal,
    /// 20 requests per 2 seconds (burst protection)
    BurstProtection,
}

impl RateLimitType {
    /// Get the window size in seconds
    fn window_seconds(&self) -> u64 {
        match self {
            RateLimitType::PostMessage => 60,
            RateLimitType::ContactReveal => 3600, // 1 hour
            RateLimitType::BurstProtection => 2,
        }
    }

    /// Get the maximum allowed requests in the window
    fn max_requests(&self) -> i64 {
        match self {
            RateLimitType::PostMessage => 1,
            RateLimitType::ContactReveal => 5,
            RateLimitType::BurstProtection => 20,
        }
    }

    /// Get the Redis key prefix
    fn key_prefix(&self) -> &str {
        match self {
            RateLimitType::PostMessage => "ratelimit:post",
            RateLimitType::ContactReveal => "ratelimit:reveal",
            RateLimitType::BurstProtection => "ratelimit:burst",
        }
    }
}

#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: i64,
    pub reset_at: u64,
}

impl RateLimiter {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Check the current rate limit status without consuming a request
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key identifying the user
    /// * `limit_type` - The type of rate limit to check
    /// 
    /// # Returns
    /// A result indicating the current rate limit status without incrementing
    pub async fn check_rate_limit_status(
        &self,
        composite_key: &str,
        limit_type: RateLimitType,
    ) -> Result<RateLimitResult> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let window_seconds = limit_type.window_seconds();
        let max_requests = limit_type.max_requests();
        let key = format!("{}:{}", limit_type.key_prefix(), composite_key);

        // Calculate the window start time
        let window_start = now - window_seconds as f64;

        // Count current requests in the window (without modifying anything)
        let current_count = self.redis
            .zcount(&key, window_start, now)
            .await
            .map_err(|e| anyhow!("Failed to count requests: {}", e))?;

        if current_count >= max_requests {
            // Rate limit exceeded
            // Get the oldest timestamp in the window to calculate when it expires
            let oldest_timestamps: Vec<(String, f64)> = self.redis
                .zrange_withscores(&key, 0, 0)
                .await
                .unwrap_or_else(|_| vec![]);
            
            let reset_at = if let Some((_, oldest_timestamp)) = oldest_timestamps.first() {
                // Reset time is when the oldest request expires (oldest_timestamp + window_seconds)
                (oldest_timestamp + window_seconds as f64) as u64
            } else {
                // Fallback: reset in full window time
                (now + window_seconds as f64) as u64
            };
            
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at,
            });
        }

        Ok(RateLimitResult {
            allowed: true,
            remaining: max_requests - current_count,
            reset_at: (now + window_seconds as f64) as u64,
        })
    }

    /// Check if a request is allowed under the rate limit
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key identifying the user
    /// * `limit_type` - The type of rate limit to check
    /// 
    /// # Returns
    /// A result indicating if the request is allowed and rate limit info
    pub async fn check_rate_limit(
        &self,
        composite_key: &str,
        limit_type: RateLimitType,
    ) -> Result<RateLimitResult> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();

        let window_seconds = limit_type.window_seconds();
        let max_requests = limit_type.max_requests();
        let key = format!("{}:{}", limit_type.key_prefix(), composite_key);

        // Calculate the window start time
        let window_start = now - window_seconds as f64;

        // Remove old entries outside the sliding window
        self.redis
            .zrembyscore(&key, 0.0, window_start)
            .await
            .map_err(|e| anyhow!("Failed to remove old entries: {}", e))?;

        // Count current requests in the window
        let current_count = self.redis
            .zcount(&key, window_start, now)
            .await
            .map_err(|e| anyhow!("Failed to count requests: {}", e))?;

        if current_count >= max_requests {
            // Rate limit exceeded
            // Get the oldest timestamp in the window to calculate when it expires
            let oldest_timestamps: Vec<(String, f64)> = self.redis
                .zrange_withscores(&key, 0, 0)
                .await
                .unwrap_or_else(|_| vec![]);
            
            let reset_at = if let Some((_, oldest_timestamp)) = oldest_timestamps.first() {
                // Reset time is when the oldest request expires (oldest_timestamp + window_seconds)
                (oldest_timestamp + window_seconds as f64) as u64
            } else {
                // Fallback: reset in full window time
                (now + window_seconds as f64) as u64
            };
            
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_at,
            });
        }

        // Add current timestamp to the sorted set
        let timestamp_str = now.to_string();
        self.redis
            .zadd(&key, now, &timestamp_str)
            .await
            .map_err(|e| anyhow!("Failed to add timestamp: {}", e))?;

        // Set expiration on the key to auto-cleanup
        self.redis
            .expire(&key, (window_seconds + 10) as i64)
            .await
            .map_err(|e| anyhow!("Failed to set expiration: {}", e))?;

        Ok(RateLimitResult {
            allowed: true,
            remaining: max_requests - current_count - 1,
            reset_at: (now + window_seconds as f64) as u64,
        })
    }

    /// Block an IP address globally for a specified duration
    /// 
    /// # Arguments
    /// * `ip` - The IP address to block
    /// * `duration_seconds` - How long to block the IP (in seconds)
    pub async fn block_ip(&self, ip: &str, duration_seconds: u64) -> Result<()> {
        let key = format!("blocked:ip:{}", ip);
        self.redis
            .set_ex(&key, "1", duration_seconds)
            .await
            .map_err(|e| anyhow!("Failed to block IP: {}", e))?;
        Ok(())
    }

    /// Check if an IP address is currently blocked
    pub async fn is_ip_blocked(&self, ip: &str) -> Result<bool> {
        let key = format!("blocked:ip:{}", ip);
        self.redis
            .exists(&key)
            .await
            .map_err(|e| anyhow!("Failed to check if IP is blocked: {}", e))
    }

    /// Get the remaining time for an IP block in seconds
    pub async fn get_ip_block_ttl(&self, ip: &str) -> Result<i64> {
        let key = format!("blocked:ip:{}", ip);
        self.redis
            .ttl(&key)
            .await
            .map_err(|e| anyhow!("Failed to get IP block TTL: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_type_values() {
        assert_eq!(RateLimitType::PostMessage.window_seconds(), 60);
        assert_eq!(RateLimitType::PostMessage.max_requests(), 1);
        
        assert_eq!(RateLimitType::ContactReveal.window_seconds(), 3600);
        assert_eq!(RateLimitType::ContactReveal.max_requests(), 5);
        
        assert_eq!(RateLimitType::BurstProtection.window_seconds(), 2);
        assert_eq!(RateLimitType::BurstProtection.max_requests(), 20);
    }
}
