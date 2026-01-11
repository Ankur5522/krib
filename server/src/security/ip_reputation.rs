use crate::redis_client::RedisClient;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};

/// Risk levels for IP reputation based on unique fingerprint reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 0-1 unique reports: 60s cooldown, full broadcast
    Level0 = 0,
    /// 2 unique reports: 300s (5m) cooldown, full broadcast
    Level1 = 1,
    /// 3-5 unique reports: 900s (15m) cooldown, shadow-throttle (only to sender's IP)
    Level2 = 2,
    /// 6+ unique reports: 7200s (2h) cooldown, hard shadowban (no broadcast)
    Level3 = 3,
}

impl RiskLevel {
    /// Get the cooldown time in seconds for this risk level
    pub fn cooldown_seconds(&self) -> u64 {
        match self {
            RiskLevel::Level0 => 60,
            RiskLevel::Level1 => 300,
            RiskLevel::Level2 => 900,
            RiskLevel::Level3 => 7200,
        }
    }

    /// Get the visibility mode for this risk level
    pub fn visibility_mode(&self) -> VisibilityMode {
        match self {
            RiskLevel::Level0 => VisibilityMode::Normal,
            RiskLevel::Level1 => VisibilityMode::Normal,
            RiskLevel::Level2 => VisibilityMode::Throttled,
            RiskLevel::Level3 => VisibilityMode::Banned,
        }
    }

    /// Determine risk level from number of unique reports
    pub fn from_report_count(count: usize) -> Self {
        match count {
            0..=1 => RiskLevel::Level0,
            2 => RiskLevel::Level1,
            3..=5 => RiskLevel::Level2,
            _ => RiskLevel::Level3,
        }
    }
}

/// Visibility mode determines how messages are broadcast
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisibilityMode {
    /// Full broadcast to all users
    Normal,
    /// Broadcast only to the sender's IP (shadow-throttle)
    Throttled,
    /// No broadcast at all (hard shadowban)
    Banned,
}

/// IP reputation manager
#[derive(Clone)]
pub struct IpReputationManager {
    redis: RedisClient,
}

impl IpReputationManager {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Add a report for a fingerprint from a specific IP
    /// Returns the new total count of unique reported fingerprints for that IP
    pub async fn add_report(&self, ip: &str, fingerprint: &str) -> Result<usize> {
        let key = format!("reports:ip:{}", ip);
        
        // Add fingerprint to the set (automatically handles duplicates)
        self.redis
            .sadd(&key, fingerprint)
            .await
            .map_err(|e| anyhow!("Failed to add report: {}", e))?;
        
        // Set expiration to 7 days - reports older than this are forgiven
        self.redis
            .expire(&key, 604800)
            .await
            .map_err(|e| anyhow!("Failed to set expiration on reports: {}", e))?;
        
        // Get the count of unique fingerprints reported from this IP
        let count = self.redis
            .scard(&key)
            .await
            .map_err(|e| anyhow!("Failed to count reports: {}", e))?;
        
        Ok(count as usize)
    }

    /// Get the number of unique reported fingerprints for an IP
    pub async fn get_report_count(&self, ip: &str) -> Result<usize> {
        let key = format!("reports:ip:{}", ip);
        let count = self.redis
            .scard(&key)
            .await
            .map_err(|e| anyhow!("Failed to get report count: {}", e))?;
        
        Ok(count as usize)
    }

    /// Get the risk level for an IP based on reports
    pub async fn get_ip_risk_level(&self, ip: &str) -> Result<RiskLevel> {
        let count = self.get_report_count(ip).await?;
        Ok(RiskLevel::from_report_count(count))
    }

    /// Check if a composite key is in cooldown and return remaining seconds
    /// Returns None if not in cooldown, Some(seconds) if still cooling down
    pub async fn check_cooldown(&self, composite_key: &str) -> Result<Option<u64>> {
        let key = format!("cooldown:{}", composite_key);
        
        match self.redis.ttl(&key).await {
            Ok(ttl) if ttl > 0 => Ok(Some(ttl as u64)),
            Ok(_) => Ok(None), // TTL <= 0 means no cooldown or expired
            Err(e) => Err(anyhow!("Failed to check cooldown: {}", e)),
        }
    }

    /// Set a cooldown for a composite key
    pub async fn set_cooldown(&self, composite_key: &str, duration_seconds: u64) -> Result<()> {
        let key = format!("cooldown:{}", composite_key);
        
        self.redis
            .set_ex(&key, "1", duration_seconds)
            .await
            .map_err(|e| anyhow!("Failed to set cooldown: {}", e))?;
        
        Ok(())
    }

    /// Check if the user can post based on cooldown
    /// Returns Ok(()) if allowed, Err with remaining seconds if in cooldown
    pub async fn check_and_update_cooldown(
        &self,
        composite_key: &str,
        ip: &str,
    ) -> Result<Result<(), u64>> {
        // Check current cooldown
        if let Some(remaining) = self.check_cooldown(composite_key).await? {
            return Ok(Err(remaining));
        }

        // Get risk level and set new cooldown
        let risk_level = self.get_ip_risk_level(ip).await?;
        let cooldown_duration = risk_level.cooldown_seconds();
        
        self.set_cooldown(composite_key, cooldown_duration).await?;
        
        Ok(Ok(()))
    }
}
