use axum::{
    extract::{Request, State, ConnectInfo},
    middleware::Next,
    response::{IntoResponse, Response},
    http::StatusCode,
};

use crate::state::AppState;
use crate::security::rate_limiter::RateLimitType;
use std::net::SocketAddr;

/// Security context extracted from request
#[derive(Clone, Debug)]
pub struct SecurityContext {
    pub composite_key: String,
    pub ip_address: String,
    pub fingerprint: String,
}

/// Extension trait to get security context from request
pub trait SecurityContextExt {
    fn security_context(&self) -> Option<&SecurityContext>;
}

/// Middleware that extracts IP and fingerprint to create composite key
/// and checks for IP blocks
pub async fn security_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    let ip_str = addr.ip().to_string();

    // Check if IP is globally blocked
    match state.rate_limiter.is_ip_blocked(&ip_str).await {
        Ok(true) => {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "IP address temporarily blocked due to excessive requests",
            ).into_response();
        }
        Err(e) => {
            eprintln!("Error checking IP block: {}", e);
            // Continue anyway - don't let Redis errors block legitimate traffic
        }
        _ => {}
    }

    // Extract fingerprint from header (sent by frontend using ThumbmarkJS)
    let fingerprint = req
        .headers()
        .get("X-Browser-Fingerprint")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();

    // Generate composite key
    let composite_key = state.key_generator.generate(&ip_str, &fingerprint);

    // Create security context
    let security_ctx = SecurityContext {
        composite_key,
        ip_address: ip_str,
        fingerprint,
    };

    // Insert security context into request extensions
    req.extensions_mut().insert(security_ctx);

    next.run(req).await
}

/// Middleware for burst protection (20 requests in 2 seconds)
pub async fn burst_protection_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Get security context from request extensions
    let security_ctx = req.extensions().get::<SecurityContext>();

    if let Some(ctx) = security_ctx {
        // Check burst protection rate limit
        match state.rate_limiter
            .check_rate_limit(&ctx.composite_key, RateLimitType::BurstProtection)
            .await
        {
            Ok(result) => {
                if !result.allowed {
                    // Block IP for 30 minutes
                    if let Err(e) = state.rate_limiter.block_ip(&ctx.ip_address, 1800).await {
                        eprintln!("Failed to block IP: {}", e);
                    }

                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        "Too many requests - IP blocked for 30 minutes",
                    ).into_response();
                }
            }
            Err(e) => {
                eprintln!("Error checking burst protection: {}", e);
                // Continue anyway
            }
        }
    }

    next.run(req).await
}
