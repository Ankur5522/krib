mod models;
mod state;
mod handlers;
mod websocket;
mod routes;
mod redis_client;
mod security;

use tower_http::cors::CorsLayer;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Load environment variables from .env file
    dotenv().ok();

    let redis_url = env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let server_secret = env::var("SERVER_SECRET")
        .unwrap_or_else(|_| {
            eprintln!("WARNING: SERVER_SECRET not set in .env, using default (NOT SECURE for production)");
            "change-this-secret-in-production".to_string()
        });

    println!("🔐 Initializing security systems...");
    let state = state::AppState::new(&redis_url, server_secret).await?;
    println!("✅ Security systems initialized");
    
    let app = routes::create_router(state)
        .layer(CorsLayer::permissive());

    println!("🚀 Server running on http://localhost:3001");
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    ).await?;
    
    Ok(())
}
