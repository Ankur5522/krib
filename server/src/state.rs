use crate::models::ChatMessage;
use crate::redis_client::RedisClient;
use crate::security::{
    CompositeKeyGenerator,
    RateLimiter,
    ShadowbanManager,
    ContentFilter,
    IpReputationManager,
    BurstProfiler,
    GovernorRateLimiter,
    ModerationService,
};
use crate::scaling::{RedisBroadcastService, MetricsTracker};
use anyhow::Result;
use std::env;

const MESSAGES_KEY: &str = "messages";
const MESSAGE_KEY_PREFIX: &str = "message:";
const MESSAGE_TTL: u64 = 172800; // 48 hours in seconds
const PUBSUB_CHANNEL: &str = "chat:messages";

#[derive(Clone)]
pub struct AppState {
    pub redis: RedisClient,
    pub key_generator: CompositeKeyGenerator,
    pub rate_limiter: RateLimiter,
    pub governor_limiter: GovernorRateLimiter,
    pub shadowban_manager: ShadowbanManager,
    pub content_filter: ContentFilter,
    pub ip_reputation: IpReputationManager,
    pub burst_profiler: BurstProfiler,
    pub broadcast: RedisBroadcastService,
    pub metrics: MetricsTracker,
    pub moderation_service: ModerationService,
}

impl AppState {
    /// Create a new AppState with Redis connection
    pub async fn new(redis_url: &str, server_secret: String) -> Result<Self> {
        let redis = RedisClient::new(redis_url).await?;
        let key_generator = CompositeKeyGenerator::new(server_secret);
        let rate_limiter = RateLimiter::new(redis.clone());
        let governor_limiter = GovernorRateLimiter::new();
        let shadowban_manager = ShadowbanManager::new(redis.clone());
        let content_filter = ContentFilter::new();
        let ip_reputation = IpReputationManager::new(redis.clone());
        let burst_profiler = BurstProfiler::new(redis.clone());
        let broadcast = RedisBroadcastService::new(redis.clone());
        let metrics = MetricsTracker::new();
        
        // Initialize moderation service with optional OpenAI API key
        let openai_api_key = env::var("OPENAI_API_KEY").ok();
        let moderation_service = ModerationService::new(openai_api_key);
        
        Ok(Self {
            redis,
            key_generator,
            rate_limiter,
            governor_limiter,
            shadowban_manager,
            content_filter,
            ip_reputation,
            burst_profiler,
            broadcast,
            metrics,
            moderation_service,
        })
    }

    /// Add a message to Redis and publish it
    pub async fn add_message(&self, message: ChatMessage) -> Result<()> {
        let message_json = serde_json::to_string(&message)?;
        
        // Store individual message with TTL
        let message_key = format!("{}{}", MESSAGE_KEY_PREFIX, message.id);
        self.redis.set_ex(&message_key, &message_json, MESSAGE_TTL).await?;
        
        // Add message ID to the sorted set (using timestamp as score)
        let timestamp = message.timestamp as f64;
        self.redis.zadd(MESSAGES_KEY, timestamp, &message.id).await?;
        
        // Set TTL on the sorted set to auto-cleanup
        self.redis.expire(MESSAGES_KEY, MESSAGE_TTL as i64).await?;
        
        // Broadcast message to all server instances via Redis Pub/Sub
        self.broadcast.broadcast_message(&message_json).await?;
        
        // Update metrics
        self.metrics.increment_messages().await;
        
        Ok(())
    }

    /// Get all messages from Redis (most recent first)
    pub async fn get_messages(&self) -> Vec<ChatMessage> {
        // Get all message IDs from sorted set (most recent first)
        let message_ids: Vec<String> = match self.redis
            .keys(&format!("{}*", MESSAGE_KEY_PREFIX))
            .await
        {
            Ok(keys) => keys,
            Err(e) => {
                eprintln!("Failed to get message keys: {}", e);
                return Vec::new();
            }
        };

        if message_ids.is_empty() {
            return Vec::new();
        }

        // Get all messages
        let mut messages = Vec::new();
        for key in message_ids {
            if let Ok(Some(json)) = self.redis.get(&key).await {
                if let Ok(msg) = serde_json::from_str::<ChatMessage>(&json) {
                    messages.push(msg);
                }
            }
        }

        // Sort by timestamp (oldest first - chronological order)
        messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        messages
    }

    /// Get a specific message by ID
    pub async fn get_message_by_id(&self, id: &str) -> Option<ChatMessage> {
        let message_key = format!("{}{}", MESSAGE_KEY_PREFIX, id);
        
        match self.redis.get(&message_key).await {
            Ok(Some(json)) => {
                serde_json::from_str(&json).ok()
            }
            _ => None
        }
    }

    /// Clean up old messages (older than TTL)
    #[allow(dead_code)]
    pub async fn cleanup_old_messages(&self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() as f64;
        
        let cutoff = now - MESSAGE_TTL as f64;
        
        // Remove old message IDs from sorted set
        self.redis.zrembyscore(MESSAGES_KEY, 0.0, cutoff).await?;
        
        Ok(())
    }

    /// Get the Redis pub/sub channel name
    pub fn get_pubsub_channel(&self) -> &str {
        PUBSUB_CHANNEL
    }
}
