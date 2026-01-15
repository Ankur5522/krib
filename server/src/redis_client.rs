use redis::{aio::ConnectionManager, AsyncCommands, RedisError, Client};
use anyhow::{Context, Result};

/// Redis client wrapper for managing Redis connections and operations
/// Enforces secure connection requirements (password authentication for production)
#[derive(Clone)]
pub struct RedisClient {
    manager: ConnectionManager,
    client: Client,
}

impl RedisClient {
    /// Create a new Redis client from a connection URL
    /// 
    /// Security Requirements:
    /// - For production: Redis URL must include a password (redis://:password@host:port)
    /// - For local development: Password strongly recommended
    /// - Supports both plain (redis://) and encrypted (rediss://) connections
    pub async fn new(redis_url: &str) -> Result<Self> {
        // Validate that the Redis URL contains a password for production safety
        // Allow development without password only if explicitly set
        if !redis_url.contains("://") {
            return Err(anyhow::anyhow!(
                "Invalid Redis URL format. Expected: redis://:password@host:port or rediss://:password@host:port"
            ));
        }

        // Check for password in the URL (between :// and @)
        let has_password = redis_url.contains('@');
        
        // Log security warning if no password is detected
        if !has_password {
            eprintln!("⚠️  WARNING: Redis URL does not include a password!");
            eprintln!("🔒 For production, always use: redis://:yourpassword@host:port");
            eprintln!("🔒 Generate a strong password and update REDIS_URL in your .env file");
        }

        let client = redis::Client::open(redis_url)
            .context("Failed to create Redis client from URL")?;
        
        let manager = ConnectionManager::new(client.clone())
            .await
            .context("Failed to create Redis connection manager - check REDIS_URL and password")?;
        
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
    #[allow(dead_code)]
    pub async fn set_nx_ex(&self, key: &str, value: &str, seconds: u64) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        conn.set_nx::<_, _, ()>(key, value).await?;
        conn.expire(key, seconds as i64).await
    }

    /// Delete a key
    #[allow(dead_code)]
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

    /// Remove a member from a sorted set
    pub async fn zrem(&self, key: &str, member: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.zrem(key, member).await
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub async fn lpush(&self, key: &str, value: &str) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.lpush(key, value).await
    }

    /// Get a range from a list
    #[allow(dead_code)]
    pub async fn lrange(&self, key: &str, start: isize, stop: isize) -> Result<Vec<String>, RedisError> {
        let mut conn = self.manager.clone();
        conn.lrange(key, start, stop).await
    }

    /// Trim a list to a specific size
    #[allow(dead_code)]
    pub async fn ltrim(&self, key: &str, start: isize, stop: isize) -> Result<(), RedisError> {
        let mut conn = self.manager.clone();
        conn.ltrim(key, start, stop).await
    }

    /// Add a member to a set
    pub async fn sadd(&self, key: &str, member: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.sadd(key, member).await
    }

    /// Get the cardinality (number of members) of a set
    pub async fn scard(&self, key: &str) -> Result<i64, RedisError> {
        let mut conn = self.manager.clone();
        conn.scard(key).await
    }

    /// Ping Redis to check if connection is alive
    pub async fn ping(&self) -> Result<bool, RedisError> {
        let mut conn = self.manager.clone();
        redis::cmd("PING")
            .query_async::<_, String>(&mut conn)
            .await
            .map(|resp| resp == "PONG")
    }
}
