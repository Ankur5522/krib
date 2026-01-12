use anyhow::Result;
use crate::redis_client::RedisClient;
use std::sync::Arc;
use tokio::sync::RwLock;

const PUBSUB_CHANNEL: &str = "chat:messages";

/// Redis Broadcast Service for horizontal scaling
/// Handles pub/sub operations to synchronize messages across multiple server instances
#[derive(Clone)]
pub struct RedisBroadcastService {
    redis: RedisClient,
}

impl RedisBroadcastService {
    pub fn new(redis: RedisClient) -> Self {
        Self { redis }
    }

    /// Publish a message to all connected server instances
    pub async fn broadcast_message(&self, message: &str) -> Result<()> {
        let mut conn = self.redis.get_client().get_async_connection().await?;
        redis::cmd("PUBLISH")
            .arg(PUBSUB_CHANNEL)
            .arg(message)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    /// Get the pub/sub channel name
    #[allow(dead_code)]
    pub fn get_channel(&self) -> &str {
        PUBSUB_CHANNEL
    }

    /// Subscribe to the broadcast channel (for WebSocket connections)
    #[allow(dead_code)]
    pub async fn subscribe(&self) -> Result<redis::aio::PubSub> {
        let conn = self.redis.get_client().get_async_connection().await?;
        Ok(conn.into_pubsub())
    }
}

/// Metrics tracker for monitoring server health and performance
#[derive(Clone)]
pub struct MetricsTracker {
    active_connections: Arc<RwLock<i64>>,
    messages_sent: Arc<RwLock<u64>>,
    contact_reveals: Arc<RwLock<u64>>,
}

impl MetricsTracker {
    pub fn new() -> Self {
        Self {
            active_connections: Arc::new(RwLock::new(0)),
            messages_sent: Arc::new(RwLock::new(0)),
            contact_reveals: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn increment_connections(&self) {
        let mut count = self.active_connections.write().await;
        *count += 1;
        metrics::gauge!("active_websocket_connections", *count as f64);
    }

    pub async fn decrement_connections(&self) {
        let mut count = self.active_connections.write().await;
        *count -= 1;
        metrics::gauge!("active_websocket_connections", *count as f64);
    }

    pub async fn increment_messages(&self) {
        let mut count = self.messages_sent.write().await;
        *count += 1;
        metrics::counter!("messages_per_second", 1);
    }

    pub async fn increment_contact_reveals(&self) {
        let mut count = self.contact_reveals.write().await;
        *count += 1;
        metrics::counter!("contact_reveals_total", 1);
    }

    pub async fn get_active_connections(&self) -> i64 {
        *self.active_connections.read().await
    }
}

/// Health check status for the server
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub redis_connected: bool,
    pub active_connections: i64,
    pub timestamp: u64,
}

impl HealthStatus {
    pub async fn check(redis: &RedisClient, metrics: &MetricsTracker) -> Self {
        let redis_connected = redis.ping().await.unwrap_or(false);
        let active_connections = metrics.get_active_connections().await;
        
        Self {
            healthy: redis_connected,
            redis_connected,
            active_connections,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}
