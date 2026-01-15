use axum::{routing::get, routing::post, Router, middleware};
use crate::{handlers, state::AppState, security::middleware::{security_middleware, burst_protection_middleware}};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/ws", get(handlers::websocket_handler))
        .route("/messages", post(handlers::post_message))
        .route("/messages", get(handlers::get_messages))
        .route("/api/contact/:message_id", get(handlers::get_contact))
        .route("/api/cooldown", get(handlers::get_cooldown))
        .route("/api/report", post(handlers::report_message))
        .route("/api/track-visitor", post(handlers::track_visitor))
        // Stats endpoints - use only burst protection, not rate limiting
        .route("/api/stats/daily", get(handlers::get_daily_stats))
        .route("/api/stats/cities", get(handlers::get_city_stats))
        .route("/health", get(handlers::health_check))
        .layer(middleware::from_fn_with_state(state.clone(), burst_protection_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), security_middleware))
        .with_state(state)
}
