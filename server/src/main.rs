mod models;
mod state;
mod handlers;
mod websocket;
mod routes;
mod redis_client;
mod security;
mod scaling;

use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use dotenvy::dotenv;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from .env file
    dotenv().ok();

    let redis_url = env::var("REDIS_URL")
        .expect("REDIS_URL must be set in .env file");
    
    let server_secret = env::var("SERVER_SECRET")
        .expect("SERVER_SECRET must be set in .env file. Generate with: openssl rand -hex 32");

    // Load CORS origin from environment
    let allowed_origin = env::var("ALLOWED_ORIGIN")
        .expect("ALLOWED_ORIGIN must be set in .env file (e.g., https://yourdomain.com)");

    println!("🔐 Initializing security systems...");
    let state = state::AppState::new(&redis_url, server_secret).await?;
    println!("✅ Security systems initialized");
    
    // Initialize Prometheus metrics exporter
    let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
    let prometheus_handle = builder
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    
    // Initialize custom metrics
    metrics::gauge!("active_websocket_connections", 0.0);
    metrics::counter!("messages_per_second", 0);
    metrics::counter!("contact_reveals_total", 0);
    
    println!("📊 Metrics initialized");
    
    // Configure CORS to only allow the specific production domain
    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origin.parse::<axum::http::HeaderValue>()
                .expect("Invalid ALLOWED_ORIGIN value")
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
            axum::http::HeaderName::from_static("x-browser-fingerprint"),
        ])
        .max_age(Duration::from_secs(3600));
    
    let app = routes::create_router(state)
        .route("/metrics", axum::routing::get(move || async move {
            prometheus_handle.render()
        }))
        .layer(TimeoutLayer::new(Duration::from_secs(30))) // 30 second timeout
        .layer(cors);

    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("🚀 Server running on http://0.0.0.0:{}", port);
    println!("📊 Metrics available at http://0.0.0.0:{}/metrics", port);
    println!("🏥 Health check available at http://0.0.0.0:{}/health", port);
    println!("🌐 CORS enabled for: {}", allowed_origin);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    // Graceful shutdown handler
    let server = axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    );
    
    // Setup graceful shutdown
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    
    println!("✅ Server ready for connections (graceful shutdown enabled)");
    
    graceful.await?;
    
    println!("👋 Server shutdown complete");
    
    Ok(())
}

/// Waits for shutdown signal (CTRL+C or SIGTERM)
async fn shutdown_signal() {
    use tokio::signal;
    
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            println!("\n🛑 Received shutdown signal (CTRL+C), shutting down gracefully...");
        },
        _ = terminate => {
            println!("\n🛑 Received SIGTERM signal, shutting down gracefully...");
        },
    }
}
