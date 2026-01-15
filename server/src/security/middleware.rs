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
#[allow(dead_code)]
pub trait SecurityContextExt {
    fn security_context(&self) -> Option<&SecurityContext>;
}

/// Middleware that extracts IP and fingerprint to create composite key
/// and checks for IP blocks
/// Handles X-Forwarded-For and Cf-Connecting-Ip headers for load balancers
pub async fn security_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Response {
    // Extract real IP from load balancer headers
    let ip_str = extract_real_ip(&req, &addr);

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

/// Extract real IP address from load balancer headers
/// Priority: Cf-Connecting-Ip > X-Forwarded-For > Direct connection
fn extract_real_ip(req: &Request, addr: &SocketAddr) -> String {
    // Check Cloudflare header first
    if let Some(cf_ip) = req.headers()
        .get("Cf-Connecting-Ip")
        .and_then(|h| h.to_str().ok())
    {
        return cf_ip.to_string();
    }

    // Check X-Forwarded-For header (standard for proxies/load balancers)
    // This header contains the original client IP when behind a proxy/load balancer
    if let Some(forwarded) = req.headers()
        .get("X-Forwarded-For")
        .and_then(|h| h.to_str().ok())
    {
        // X-Forwarded-For can be comma-separated, take the first (original client)
        if let Some(first_ip) = forwarded.split(',').next() {
            return first_ip.trim().to_string();
        }
    }

    // Fallback to direct connection IP
    addr.ip().to_string()
}

/// Middleware for burst protection (20 requests in 2 seconds)
/// Also includes burst profiler to detect bot-like behavior (5 endpoints in 500ms)
/// Enforces IP rate limiting with governor (50 requests per minute per IP)
pub async fn burst_protection_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Response {
    // Get security context from request extensions
    let security_ctx = req.extensions().get::<SecurityContext>();
    let uri_path = req.uri().path().to_string();
    let method = req.method().clone();

    // Skip rate limiting for read-only stats endpoints
    let is_stats_endpoint = uri_path.starts_with("/api/stats/") 
        || uri_path == "/health"
        || uri_path == "/api/cooldown";
    let is_get_request = method == axum::http::Method::GET;

    if let Some(ctx) = security_ctx {
        // Check governor-based IP rate limiting (50 requests per minute)
        // Skip for stats endpoints and GET requests (read-only, harmless)
        if !is_stats_endpoint && !is_get_request && !state.governor_limiter.check_ip_rate_limit(&ctx.ip_address) {
            eprintln!("🚫 IP rate limit exceeded for: {}", ctx.ip_address);
            return (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded: 50 requests per minute per IP",
            ).into_response();
        }

        // Check burst profiler for bot detection - skip for GET requests (harmless reads)
        if !is_get_request {
            match state.burst_profiler.check_burst(&ctx.composite_key, &uri_path).await {
                Ok(true) => {
                    // Bot detected - shadowban immediately
                    eprintln!("🤖 Bot detected via burst profiler: {}", ctx.composite_key);
                    
                    if let Err(e) = state.shadowban_manager.shadowban(
                        &ctx.composite_key,
                        Some("Bot detected - burst pattern"),
                        Some(86400), // 24 hour ban
                    ).await {
                        eprintln!("Failed to shadowban bot: {}", e);
                    }

                    // Also block the IP
                    if let Err(e) = state.rate_limiter.block_ip(&ctx.ip_address, 1800).await {
                        eprintln!("Failed to block IP: {}", e);
                    }

                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        "Suspicious activity detected",
                    ).into_response();
                }
                Err(e) => {
                    eprintln!("Error checking burst profiler: {}", e);
                }
                _ => {}
            }
        }

        // Check burst protection rate limit (20 requests in 2 seconds)
        // Skip for stats endpoints and GET requests (read-only, harmless)
        if !is_stats_endpoint && !is_get_request {
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
    }

    next.run(req).await
}
