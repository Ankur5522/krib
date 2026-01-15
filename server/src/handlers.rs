use axum::{
    extract::{ws::WebSocketUpgrade, State, Path, Query},
    http::StatusCode,
    response::Response,
    Json, Extension,
};
use serde_json::json;
use crate::{
    models::{ChatMessage, PostMessageRequest, RateLimitError, ContentFilterError, ReportMessageRequest, ReportResponse},
    state::AppState,
    websocket::handle_websocket,
    security::middleware::SecurityContext,
    security::rate_limiter::RateLimitType,
};

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

pub async fn post_message(
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
    Json(request): Json<PostMessageRequest>,
) -> Result<Json<ChatMessage>, (StatusCode, Json<serde_json::Value>)> {
    // Check honeypot field
    let honeypot_result = state.content_filter.check_honeypot(request.website.as_deref());
    if !honeypot_result.is_allowed {
        // Hard block the composite key permanently
        if let Err(e) = state.shadowban_manager.shadowban(
            &security_ctx.composite_key,
            Some("Honeypot triggered - bot detected"),
            None, // Permanent
        ).await {
            eprintln!("Failed to shadowban honeypot violator: {}", e);
        }

        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ContentFilterError::new(
                honeypot_result.reason.unwrap_or_else(|| "Bot detected".to_string())
            )))
        ));
    }

    // Check if user is shadowbanned
    let is_shadowbanned = state.shadowban_manager
        .is_shadowbanned(&security_ctx.composite_key)
        .await
        .unwrap_or(false);

    // Also check if fingerprint is shadowbanned due to reports
    let reported_key = format!("reported:{}", security_ctx.fingerprint);
    let is_reported_shadowbanned = state.shadowban_manager
        .is_shadowbanned(&reported_key)
        .await
        .unwrap_or(false);

    let is_shadowbanned_total = is_shadowbanned || is_reported_shadowbanned;

    // Validate message length
    if request.message.len() > 280 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Message too long (max 280 characters)"}))
        ));
    }

    if request.message.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Message cannot be empty"}))
        ));
    }

    // Check content filters
    let filter_result = state.content_filter.check_message(&request.message);
    if !filter_result.is_allowed {
        // Increment violation count
        if let Ok(violations) = state.shadowban_manager
            .increment_violations(&security_ctx.composite_key)
            .await
        {
            // Auto-shadowban after 3 violations (24 hour ban)
            let _ = state.shadowban_manager
                .auto_shadowban_on_violations(&security_ctx.composite_key, 3, 86400)
                .await;
            
            eprintln!("Content violation by {}: {} violations", security_ctx.composite_key, violations);
        }

        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ContentFilterError::new(
                filter_result.reason.unwrap_or_else(|| "Content policy violation".to_string())
            )))
        ));
    }

    // Run comprehensive moderation checks (profanity, relevance, spam, OpenAI)
    let moderation_result = state.moderation_service.moderate_message(&request.message).await;
    if !moderation_result.is_allowed {
        // Increment violation count for moderation violations
        if let Ok(violations) = state.shadowban_manager
            .increment_violations(&security_ctx.composite_key)
            .await
        {
            // Auto-shadowban after 3 violations (24 hour ban)
            let _ = state.shadowban_manager
                .auto_shadowban_on_violations(&security_ctx.composite_key, 3, 86400)
                .await;
            
            eprintln!("Moderation violation by {}: {} - {} violations", 
                     security_ctx.composite_key, 
                     moderation_result.reason.as_ref().unwrap_or(&"Unknown violation".to_string()),
                     violations);
        }

        return Err((
            StatusCode::FORBIDDEN,
            Json(json!(ContentFilterError::new(
                moderation_result.reason.unwrap_or_else(|| "Content policy violation".to_string())
            )))
        ));
    }

    // Validate phone number format if provided
    if !state.content_filter.validate_phone(request.phone.as_deref()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid phone number format"}))
        ));
    }

    // Check suspicious patterns
    if state.content_filter.is_suspicious_pattern(&request.message) {
        // Increment violations for suspicious patterns
        let _ = state.shadowban_manager
            .increment_violations(&security_ctx.composite_key)
            .await;
    }

    // Check IP reputation risk level and apply cooldowns based on it
    let ip_risk_level = state.ip_reputation
        .get_ip_risk_level(&security_ctx.ip_address)
        .await
        .unwrap_or(crate::security::ip_reputation::RiskLevel::Level0);
    
    let visibility_mode = ip_risk_level.visibility_mode();
    
    // Check if IP is in cooldown based on risk level
    if let Ok(Some(remaining)) = state.ip_reputation
        .check_cooldown(&security_ctx.composite_key)
        .await
    {
        // IP is in cooldown - return error with remaining seconds
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!(RateLimitError::new(remaining)))
        ));
    }

    // Check rate limit for posting
    let rate_limit_result = state.rate_limiter
        .check_rate_limit(&security_ctx.composite_key, RateLimitType::PostMessage)
        .await
        .map_err(|e| {
            eprintln!("Rate limit check error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check rate limit"}))
            )
        })?;

    if !rate_limit_result.allowed {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!(RateLimitError::new(rate_limit_result.reset_at)))
        ));
    }
    
    // Set cooldown for the composite key based on risk level
    let cooldown_duration = ip_risk_level.cooldown_seconds();
    if let Err(e) = state.ip_reputation
        .set_cooldown(&security_ctx.composite_key, cooldown_duration)
        .await
    {
        eprintln!("Failed to set IP reputation cooldown: {}", e);
    }

    let message = ChatMessage::new(
        request.browser_id,
        request.message,
        request.message_type,
        request.phone,
        request.location,
    );

    // Check IP reputation visibility restrictions
    use crate::security::ip_reputation::VisibilityMode;
    
    // If shadowbanned or visibility is banned, pretend to succeed but don't broadcast
    if is_shadowbanned_total || visibility_mode == VisibilityMode::Banned {
        // Just return success without storing/broadcasting
        return Ok(Json(message));
    }

    // Normal flow: add message to Redis and broadcast via pub/sub
    state.add_message(message.clone())
        .await
        .map_err(|e| {
            eprintln!("Failed to add message: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to post message"}))
            )
        })?;

    // Track message count (using Redis increment for today)
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let message_count_key = format!("stats:message_count:{}", today);
    if let Err(e) = state.redis.incr(&message_count_key).await {
        eprintln!("Failed to increment message count: {}", e);
    }
    // Set expiration to 7 days
    let _ = state.redis.expire(&message_count_key, 604800).await;

    Ok(Json(message))
}

use std::collections::HashMap;

pub async fn get_messages(
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<ChatMessage>> {
    let location_filter = params.get("location");
    
    // Track unique daily visitors per city (not just page views)
    if let Some(city) = location_filter {
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        // Use a Redis set to track unique visitors per city per day
        // Key format: stats:city_visitors:CITY:DATE
        let city_visitors_key = format!("stats:city_visitors:{}:{}", city, today);
        
        // Add the fingerprint to the set (returns 1 if new, 0 if already exists)
        match state.redis.sadd(&city_visitors_key, &security_ctx.fingerprint).await {
            Ok(is_new) => {
                // Only increment if this is a new visitor today
                if is_new > 0 {
                    let city_views_key = format!("stats:city_views:{}:{}", city, today);
                    if let Err(e) = state.redis.incr(&city_views_key).await {
                        eprintln!("Failed to increment city views for {}: {}", city, e);
                    }
                    // Set expiry to 7 days for both keys
                    let _ = state.redis.expire(&city_views_key, 604800).await;
                }
                // Always set expiry on the visitors set
                let _ = state.redis.expire(&city_visitors_key, 604800).await;
            }
            Err(e) => {
                eprintln!("Failed to track visitor for {}: {}", city, e);
            }
        }
    }
    
    let messages = state.get_messages()
        .await
        .into_iter()
        .filter(|msg| {
            // If location filter is provided, only include messages with matching location
            if let Some(filter_location) = location_filter {
                msg.location.as_ref().map_or(false, |loc| loc == filter_location)
            } else {
                true
            }
        })
        .map(|mut msg| {
            msg.phone = None;
            msg
        })
        .collect();
    Json(messages)
}

pub async fn get_contact(
    Path(message_id): Path<String>,
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // Check rate limit for contact reveal (5 per hour)
    let rate_limit_result = state.rate_limiter
        .check_rate_limit(&security_ctx.composite_key, RateLimitType::ContactReveal)
        .await
        .map_err(|e| {
            eprintln!("Rate limit check error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check rate limit"}))
            )
        })?;

    if !rate_limit_result.allowed {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!(RateLimitError::new(rate_limit_result.reset_at)))
        ));
    }

    match state.get_message_by_id(&message_id).await {
        Some(message) => {
            if let Some(phone) = message.phone {
                // Update contact reveal metric
                state.metrics.increment_contact_reveals().await;
                
                Ok(Json(json!({ "phone": phone })))
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "No contact information available"}))
                ))
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Message not found"}))
        ))
    }
}

pub async fn get_cooldown(
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
) -> Json<serde_json::Value> {
    // Check current rate limit status without incrementing
    let rate_limit_result = state.rate_limiter
        .check_rate_limit_status(&security_ctx.composite_key, RateLimitType::PostMessage)
        .await;

    match rate_limit_result {
        Ok(result) => {
            if result.allowed {
                Json(json!({
                    "can_post": true,
                    "remaining_seconds": 0
                }))
            } else {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let seconds_remaining = if result.reset_at > now {
                    result.reset_at - now
                } else {
                    0
                };
                Json(json!({
                    "can_post": false,
                    "remaining_seconds": seconds_remaining
                }))
            }
        }
        Err(_) => {
            Json(json!({
                "can_post": true,
                "remaining_seconds": 0
            }))
        }
    }
}

pub async fn report_message(
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
    Json(request): Json<ReportMessageRequest>,
) -> Result<Json<ReportResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Verify the message exists
    let message = state.get_message_by_id(&request.message_id).await;
    if message.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Message not found"}))
        ));
    }

    let message = message.unwrap();

    // Verify the browser_id matches
    if message.browser_id != request.reported_browser_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid browser ID"}))
        ));
    }

    // Can't report your own messages
    if message.browser_id == security_ctx.fingerprint {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Cannot report your own message"}))
        ));
    }

    // Add the report to IP reputation system
    // Track reports both per fingerprint and per IP address
    // Note: We only have the reporting user's IP, not the reported user's IP
    // So we track fingerprint-based reports to the IP reputation system
    
    // First, add report to IP reputation for the reporting user's IP and the reported fingerprint
    let _ip_report_count = state.ip_reputation
        .add_report(&security_ctx.ip_address, &request.reported_browser_id)
        .await
        .unwrap_or(0);
    
    // For 3 reports on a fingerprint, shadowban that fingerprint
    let report_key = format!("reports:fingerprint:{}", request.reported_browser_id);
    let report_count = match state.redis.incr(&report_key).await {
        Ok(count) => count,
        Err(e) => {
            eprintln!("Failed to increment report count: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to process report"}))
            ));
        }
    };

    // Set expiration on reports (forgive after 7 days)
    let _ = state.redis.expire(&report_key, 604800).await;

    // If 5 or more reports, delete the message
    if report_count >= 5 {
        if let Err(e) = state.delete_message(&request.message_id).await {
            eprintln!("Failed to delete reported message {}: {}", request.message_id, e);
        } else {
            eprintln!("Message {} deleted after {} reports", request.message_id, report_count);
        }
    }

    // If 3 or more reports, shadowban the fingerprint permanently
    if report_count >= 3 {
        // Create a composite key for the reported user (we use fingerprint as basis)
        let reported_composite_key = format!("reported:{}", request.reported_browser_id);
        
        if let Err(e) = state.shadowban_manager.shadowban(
            &reported_composite_key,
            Some(&format!("Auto-shadowbanned after {} reports", report_count)),
            None, // Permanent shadowban
        ).await {
            eprintln!("Failed to shadowban reported user: {}", e);
        }
    }

    Ok(Json(ReportResponse {
        success: true,
        message: "Report submitted successfully".to_string(),
        reports_on_ip: report_count as usize,
    }))
}

/// Health check endpoint for load balancer
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Use the scaling health check
    let health = crate::scaling::HealthStatus::check(&state.redis, &state.metrics).await;
    
    if health.healthy {
        Ok(Json(serde_json::to_value(health).unwrap()))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

/// Track a unique visitor by IP address
pub async fn track_visitor(
    State(state): State<AppState>,
    Extension(security_ctx): Extension<SecurityContext>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get today's date in YYYY-MM-DD format (UTC)
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    // Track unique IPs (using a Redis set for today)
    let unique_ips_key = format!("stats:unique_ips:{}", today);
    if let Err(e) = state.redis.sadd(&unique_ips_key, &security_ctx.ip_address).await {
        eprintln!("Failed to track visitor IP: {}", e);
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    
    // Set expiration to 7 days (to clean up old stats)
    if let Err(e) = state.redis.expire(&unique_ips_key, 604800).await {
        eprintln!("Failed to set expiration on unique IPs: {}", e);
    }
    
    Ok(Json(json!({
        "success": true,
        "message": "Visitor tracked"
    })))
}

/// Get daily statistics (unique IPs and message count for the day)
pub async fn get_daily_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get today's date in YYYY-MM-DD format (UTC)
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    // Get unique IPs for today
    let unique_ips_key = format!("stats:unique_ips:{}", today);
    let unique_ips = state.redis
        .scard(&unique_ips_key)
        .await
        .unwrap_or(0) as u64;
    
    // Get message count for today
    let message_count_key = format!("stats:message_count:{}", today);
    let message_count = state.redis
        .get(&message_count_key)
        .await
        .ok()
        .flatten()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);
    
    Ok(Json(json!({
        "unique_ips": unique_ips,
        "message_count": message_count,
    })))
}
/// Get city-wise daily views
/// Returns average daily views for major cities
pub async fn get_city_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    
    // List of major cities to track
    let major_cities = vec![
        "Bengaluru", "Hyderabad", "Pune", "Chennai", "Kolkata",
        "Thiruvananthapuram", "Delhi", "Noida", "Gurgaon",
    ];
    
    let mut city_stats = Vec::new();
    
    for city in major_cities {
        // Track views per city per day
        let city_views_key = format!("stats:city_views:{}:{}", city, today);
        let views: u64 = state.redis
            .get(&city_views_key)
            .await
            .ok()
            .flatten()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);
        
        // Calculate daily average (for now, just today's count)
        // In future, can calculate average over last 7 days
        city_stats.push(json!({
            "city": city,
            "views": views,
            "daily_average": views,
        }));
    }
    
    // Sort by views descending
    let mut city_stats_vec: Vec<serde_json::Value> = city_stats;
    city_stats_vec.sort_by(|a, b| {
        let views_a = a.get("views").and_then(|v| v.as_u64()).unwrap_or(0);
        let views_b = b.get("views").and_then(|v| v.as_u64()).unwrap_or(0);
        views_b.cmp(&views_a)
    });
    
    Ok(Json(serde_json::json!(city_stats_vec)))
}