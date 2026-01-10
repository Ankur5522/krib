use redis::{aio::ConnectionManager, AsyncCommands, RedisError, Client};
use anyhow::{Context, Result};

/// Redis client wrapper for managing Redis connections and operations
#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
    client: Client,
}

impl RedisClient {
    /// Create a new Redis client from a connection URL
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client")?;
        
        let manager = ConnectionManager::new(client.clone())
            .await
            .context("Failed to create Redis connection manager")?;
        
        Ok(Self { manager, client })
    }

    /// Get the underlying client for pub/sub operations
    pub fn get_client(&self) -> Client {
        self.client.clone()
    }

    /// Set a key-value pair with an expiration time (in seconds)
    pub async fn set_ex(&self, key: &str, value: &str, seconds: u64) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.set_ex(key, value, seconds).await
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>, RedisError> {
        let mut conn = self.manager.clone();
        conn.get(key).await
    }

    /// Increment a key and return the new value
    pub async fn incr(&self, key: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.incr(key, 1).await
    }

    /// Set key with expiration if it doesn't exist
    pub async fn set_nx_ex(&self, key: &str, value: &str, seconds: u64) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.set_nx::<_, _, ()>(key, value).await?;
        conn.expire(key, seconds as i64).await
    }

    /// Delete a key
    pub async fn del(&self, key: &str) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.del(key).await
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.exists(key).await
    }

    /// Get the time-to-live (TTL) of a key in seconds
    pub async fn ttl(&self, key: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.ttl(key).await
    }

    /// Add an element to a sorted set with a score (for sliding window)
    pub async fn zadd(&self, key: &str, score: f64, member: &str) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.zadd(key, member, score).await
    }

    /// Remove elements from a sorted set by score range
    pub async fn zrembyscore(&self, key: &str, min: f64, max: f64) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.zrembyscore(key, min, max).await
    }

    /// Count elements in a sorted set within a score range
    pub async fn zcount(&self, key: &str, min: f64, max: f64) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.zcount(key, min, max).await
    }

    /// Get a range from sorted set with scores
    pub async fn zrange_withscores(&self, key: &str, start: isize, stop: isize) -> Result<Vec<(String, f64)>, RedisError> {
        let mut conn = self.manager.clone();
        redis::cmd("ZRANGE")
            .arg(key)
            .arg(start)
            .arg(stop)
            .arg("WITHSCORES")
            .query_async(&mut conn)
            .await
    }

    /// Set expiration on a key
    pub async fn expire(&self, key: &str, seconds: i64) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.expire(key, seconds).await
    }

    /// Get multiple values by keys
    pub async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<String>>, RedisError> {
        let mut conn = self.manager.clone();
        conn.get(keys).await
    }

    /// Get all keys matching a pattern
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>, RedisError> {
        let mut conn = self.manager.clone();
        redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
    }

    /// Add to a list (left push)
    pub async fn lpush(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.lpush(key, value).await
    }

    /// Get a range from a list
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>, RedisError> {
        let mut conn = self.manager.clone();
        conn.lrange(key, start, stop).await
    }

    /// Trim a list to a specific size
    pub async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.ltrim(key, start, stop).await
    }
}
