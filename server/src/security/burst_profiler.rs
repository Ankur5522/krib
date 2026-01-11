use anyhow::Result;
use crate::redis_client::RedisClient;
use std::time::{SystemTime, UNIX_EPOCH};

const BURST_WINDOW_MS: u64 = 500; // 500ms window
const BURST_THRESHOLD: usize = 5; // 5 different endpoints

/// Burst Profiler for detecting bot-like behavior
/// Tracks endpoint access patterns to identify bots hitting multiple endpoints rapidly
#[derive(Clone)]
pub struct BurstProfiler {
    redis: RedisClient,
}

impl BurstProfiler {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Record an endpoint access and check if it triggers burst detection
    /// Returns true if the access pattern is suspicious (likely a bot)
    pub async fn check_burst(&self, composite_key: &str, endpoint: &str) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis() as u64;
        
        let burst_key = format!("burst:{}", composite_key);
        
        // Add current endpoint with timestamp score
        self.redis.zadd(&burst_key, now as f64, endpoint).await?;
        
        // Remove old entries outside the burst window
        let window_start = now.saturating_sub(BURST_WINDOW_MS);
        self.redis.zrembyscore(&burst_key, 0.0, window_start as f64).await?;
        
        // Set TTL on burst key (cleanup after 1 minute of inactivity)
        self.redis.expire(&burst_key, 60).await?;
        
        // Get unique endpoints hit in the burst window
        let entries = self.redis.zrange_withscores(&burst_key, 0, -1).await?;
        
        // Count unique endpoints
        let unique_endpoints: std::collections::HashSet<String> = 
            entries.iter().map(|(endpoint, _)| endpoint.clone()).collect();
        
        // If user hits 5+ different endpoints in under 500ms, flag as bot
        if unique_endpoints.len() >= BURST_THRESHOLD {
            eprintln!(
                "🚨 Burst detection triggered for {}: {} unique endpoints in {}ms",
                composite_key,
                unique_endpoints.len(),
                BURST_WINDOW_MS
            );
            return Ok(true);
        }
        
        Ok(false)
    }

    /// Get burst statistics for a composite key
    pub async fn get_burst_stats(&self, composite_key: &str) -> Result<BurstStats> {
        let burst_key = format!("burst:{}", composite_key);
        let entries = self.redis.zrange_withscores(&burst_key, 0, -1).await?;
        
        let unique_endpoints: std::collections::HashSet<String> = 
            entries.iter().map(|(endpoint, _)| endpoint.clone()).collect();
        
        let total_requests = entries.len();
        let unique_endpoints_count = unique_endpoints.len();
        
        Ok(BurstStats {
            total_requests,
            unique_endpoints_count,
            is_suspicious: unique_endpoints_count >= BURST_THRESHOLD,
        })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BurstStats {
    pub total_requests: usize,
    pub unique_endpoints_count: usize,
    pub is_suspicious: bool,
}
