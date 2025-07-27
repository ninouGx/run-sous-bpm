use axum::{
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;
use tracing::{info, Level};
use tracing_subscriber;

async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "run-sous-bpm-api",
            "timestamp": chrono::Utc::now().to_rfc3339()
        }))
    )
}

async fn root() -> Json<Value> {
    Json(json!({
        "message": "Welcome to Run Sous BPM API",
        "version": "0.1.0"
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Build our application with routes
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .layer(CorsLayer::permissive());

    // Determine port
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    info!("Run Sous BPM API server starting on port {}", port);
    
    // Run the server
    axum::serve(listener, app).await?;

    Ok(())
}