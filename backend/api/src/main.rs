mod handlers;

use std::sync::Arc;
use axum::{ routing::get, Router, Extension };
use run_sous_bpm_core::services::OAuthSessionManager;
use tower_http::cors::CorsLayer;
use tracing::{ info, Level };
use tracing_subscriber;
use handlers::{ root, health, handler_404, oauth_callback, oauth_process_callback };

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let session_store = Arc::new(OAuthSessionManager::new());
    // Load environment variables
    dotenvy::dotenv().ok();
    let oauth_callback_route = std::env
        ::var("REDIRECT_ENDPOINT")
        .unwrap_or_else(|_| "/api/oauth/callback".to_string());

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/oauth/{provider}/authorize", get(oauth_callback))
        .route(&oauth_callback_route, get(oauth_process_callback))
        .layer(Extension(session_store))
        .layer(CorsLayer::permissive())
        .fallback(handler_404);

    // Determine port
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    info!("Run Sous BPM API server starting on port {}", port);
    info!("Available at:");
    info!("\t- http://localhost:{}/", port);
    info!("\t- http://localhost:{}/health", port);
    info!("\t- http://localhost:{}/api/oauth/strava/authorize", port);
    info!("\t- http://localhost:{}/api/oauth/spotify/authorize", port);

    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}
