mod handlers;
mod middleware;

use axum::{
    Extension, Router,
    middleware::from_fn,
    routing::{get, post},
};
use axum_login::{AuthManagerLayerBuilder, login_required, tower_sessions::MemoryStore};
use handlers::{
    handler_404, health, login_user, logout_user, oauth_callback, oauth_process_callback,
    register_user, root,
};
use run_sous_bpm_core::{
    auth::AuthBackend, database::establish_db_connection, services::OAuthSessionManager,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time},
};
use tracing::{Level, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_line_number(true)
        .pretty()
        .init();

    // Session store for OAuth (PKCE, CSRF, etc.)
    let oauth_session_store = Arc::new(OAuthSessionManager::new());
    // Database connection
    let db_connection = establish_db_connection().await?;
    // Axum Login layers
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("run_sous_bpm_session")
        .with_secure(false) // Set to true in production with HTTPS
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));
    let auth_backend = AuthBackend::new(db_connection.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    dotenvy::dotenv().ok();
    let oauth_callback_route =
        std::env::var("REDIRECT_ENDPOINT").unwrap_or_else(|_| "/api/oauth/callback".to_string());

    let public_routes = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        .route(&oauth_callback_route, get(oauth_process_callback));

    let protected_routes = Router::new()
        .route("/api/oauth/{provider}/authorize", get(oauth_callback))
        .route("/api/auth/logout", post(logout_user))
        .route_layer(login_required!(AuthBackend));

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(from_fn(middleware::handle_errors))
        .layer(CorsLayer::permissive())
        .layer(auth_layer)
        .layer(Extension(db_connection))
        .layer(Extension(oauth_session_store))
        .fallback(handler_404);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Run Sous BPM API server starting on port {}", port);
    info!("Available at:");
    info!("\t- http://localhost:{port}/");
    info!("\t- http://localhost:{port}/health");
    info!("\t- http://localhost:{port}/api/oauth/strava/authorize");
    info!("\t- http://localhost:{port}/api/oauth/spotify/authorize");
    info!("\t- http://localhost:{port}/api/auth/register");
    info!("\t- http://localhost:{port}/api/auth/login");
    info!("\t- http://localhost:{port}/api/auth/logout");

    axum::serve(listener, app).await?;

    Ok(())
}
