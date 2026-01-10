use axum::{
    extract::{ws::WebSocketUpgrade, State, Path, Query},
    http::StatusCode,
    response::Response,
    Json, Extension,
};
use serde_json::json;
use crate::{
    models::{ChatMessage, PostMessageRequest, RateLimitError, ContentFilterError},
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

    let message = ChatMessage::new(
        request.browser_id,
        request.message,
        request.message_type,
        request.phone,
        request.location,
    );

    // If shadowbanned, pretend to succeed but don't broadcast
    if is_shadowbanned {
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

    Ok(Json(message))
}

use std::collections::HashMap;

pub async fn get_messages(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<ChatMessage>> {
    let location_filter = params.get("location");
    
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
        .check_rate_limit(&security_ctx.composite_key, RateLimitType::PostMessage)
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
