use crate::redis_client::RedisClient;
use anyhow::{Result, anyhow};

/// Manages shadowban functionality for users
/// Shadowbanned users can send messages, but they are not broadcast to others
#[derive(Clone)]
pub struct ShadowbanManager {
    redis: RedisClient,
}

impl ShadowbanManager {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Check if a composite key is shadowbanned
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key to check
    /// 
    /// # Returns
    /// True if the user is shadowbanned, false otherwise
    pub async fn is_shadowbanned(&self, composite_key: &str) -> Result<bool> {
        let key = format!("shadowban:{}", composite_key);
        self.redis
            .exists(&key)
            .await
            .map_err(|e| anyhow!("Failed to check shadowban status: {}", e))
    }

    /// Shadowban a user by their composite key
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key to shadowban
    /// * `reason` - Optional reason for the shadowban (for admin tracking)
    /// * `duration_seconds` - Optional duration in seconds (None = permanent)
    pub async fn shadowban(
        &self,
        composite_key: &str,
        reason: Option<&str>,
        duration_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("shadowban:{}", composite_key);
        let value = reason.unwrap_or("no_reason");

        match duration_seconds {
            Some(duration) => {
                self.redis
                    .set_ex(&key, value, duration)
                    .await
                    .map_err(|e| anyhow!("Failed to shadowban user: {}", e))?;
            }
            None => {
                // Permanent shadowban (set with a very long expiration, e.g., 10 years)
                self.redis
                    .set_ex(&key, value, 315360000) // ~10 years
                    .await
                    .map_err(|e| anyhow!("Failed to shadowban user: {}", e))?;
            }
        }

        Ok(())
    }

    /// Remove a shadowban from a user
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key to un-shadowban
    #[allow(dead_code)]
    pub async fn remove_shadowban(&self, composite_key: &str) -> Result<()> {
        let key = format!("shadowban:{}", composite_key);
        self.redis
            .del(&key)
            .await
            .map_err(|e| anyhow!("Failed to remove shadowban: {}", e))?;
        Ok(())
    }

    /// Get the reason for a shadowban (if available)
    #[allow(dead_code)]
    pub async fn get_shadowban_reason(&self, composite_key: &str) -> Result<Option<String>> {
        let key = format!("shadowban:{}", composite_key);
        self.redis
            .get(&key)
            .await
            .map_err(|e| anyhow!("Failed to get shadowban reason: {}", e))
    }

    /// Get the time-to-live for a shadowban in seconds
    /// Returns -1 for permanent bans, -2 if key doesn't exist
    #[allow(dead_code)]
    pub async fn get_shadowban_ttl(&self, composite_key: &str) -> Result<i64> {
        let key = format!("shadowban:{}", composite_key);
        self.redis
            .ttl(&key)
            .await
            .map_err(|e| anyhow!("Failed to get shadowban TTL: {}", e))
    }

    /// Increment the violation count for a composite key
    /// This can be used to auto-shadowban after a certain number of violations
    pub async fn increment_violations(&self, composite_key: &str) -> Result<i64> {
        let key = format!("violations:{}", composite_key);
        let count = self.redis
            .incr(&key)
            .await
            .map_err(|e| anyhow!("Failed to increment violations: {}", e))?;
        
        // Set expiration for violations counter (e.g., reset after 24 hours)
        self.redis
            .expire(&key, 86400)
            .await
            .map_err(|e| anyhow!("Failed to set expiration on violations: {}", e))?;
        
        Ok(count)
    }

    /// Get the current violation count for a composite key
    pub async fn get_violations(&self, composite_key: &str) -> Result<i64> {
        let key = format!("violations:{}", composite_key);
        match self.redis.get(&key).await {
            Ok(Some(count_str)) => {
                count_str.parse::<i64>()
                    .map_err(|e| anyhow!("Failed to parse violation count: {}", e))
            }
            Ok(None) => Ok(0),
            Err(e) => Err(anyhow!("Failed to get violations: {}", e)),
        }
    }

    /// Auto-shadowban if violations exceed a threshold
    /// 
    /// # Arguments
    /// * `composite_key` - The composite key to check
    /// * `threshold` - Number of violations before auto-shadowban
    /// * `duration_seconds` - Duration of the auto-shadowban
    /// 
    /// # Returns
    /// True if user was shadowbanned, false otherwise
    pub async fn auto_shadowban_on_violations(
        &self,
        composite_key: &str,
        threshold: i64,
        duration_seconds: u64,
    ) -> Result<bool> {
        let violations = self.get_violations(composite_key).await?;
        
        if violations >= threshold {
            self.shadowban(
                composite_key,
                Some(&format!("Auto-banned: {} violations", violations)),
                Some(duration_seconds),
            ).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
