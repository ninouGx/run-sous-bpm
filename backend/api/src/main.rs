mod handlers;
mod middleware;
mod responses;

use axum::http::{HeaderValue, Method};
use axum::{
    Router,
    middleware::from_fn,
    routing::{get, patch, post},
};
use axum_login::{AuthManagerLayerBuilder, login_required, tower_sessions::MemoryStore};
use handlers::{
    get_activity_music, get_current_user, get_strava_activities, get_strava_activity_streams,
    handler_404, health, login_user, logout_user, oauth_callback, oauth_process_callback,
    register_user, root, sync_all_strava_activity_streams, sync_strava_activities,
    sync_strava_activity_streams,
};
use run_sous_bpm_core::{
    auth::AuthBackend, database::establish_db_connection, services::OAuthSessionManager,
};
use run_sous_bpm_integrations::{
    common::{AuthenticatedClient, IntegrationClient},
    strava::StravaApiClient,
};
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_sessions::{
    Expiry, SessionManagerLayer,
    cookie::{SameSite, time},
};
use tracing::{Level, info};

use crate::handlers::{patch_user, remove_oauth_provider};

#[derive(Clone)]
struct AppState {
    db_connection: DatabaseConnection,
    oauth_session_store: Arc<OAuthSessionManager>,
    strava_client: Arc<StravaApiClient>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(true)
        .with_line_number(true)
        .pretty()
        .init();

    let oauth_session_store = Arc::new(OAuthSessionManager::new());
    let db_connection = establish_db_connection().await?;

    let http_client = Arc::new(AuthenticatedClient::new());

    let strava_base_url = std::env::var("STRAVA_API_URL")
        .unwrap_or_else(|_| "https://www.strava.com/api/v3".to_string());
    let strava_integration_client = IntegrationClient::new(http_client.clone());
    let strava_client = Arc::new(StravaApiClient::new(
        strava_integration_client,
        strava_base_url,
    ));

    let state = AppState {
        db_connection: db_connection.clone(),
        oauth_session_store: oauth_session_store.clone(),
        strava_client,
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("run_sous_bpm_session")
        .with_secure(false)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));
    let auth_backend = AuthBackend::new(db_connection.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let oauth_callback_route =
        std::env::var("REDIRECT_ENDPOINT").unwrap_or_else(|_| "/api/oauth/callback".to_string());

    // ====== CONFIGURATION CORS ======
    let allowed_origin =
        std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");

    let cors = CorsLayer::new()
        .allow_origin(allowed_origin.parse::<HeaderValue>().expect("Invalid CORS origin"))
        .allow_credentials(true)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);
    // ================================

    let public_routes = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/auth/register", post(register_user))
        .route("/api/auth/login", post(login_user))
        .route(&oauth_callback_route, get(oauth_process_callback));

    let protected_routes = Router::new()
        .route("/api/auth/me", get(get_current_user))
        .route("/api/auth/logout", post(logout_user))
        .route("/api/user", patch(patch_user))
        .route("/api/oauth/{provider}/authorize", get(oauth_callback))
        .route(
            "/api/oauth/{provider}/disconnect",
            post(remove_oauth_provider),
        )
        .route("/api/strava/activities", get(get_strava_activities))
        .route(
            "/api/strava/activities/{id}/streams",
            get(get_strava_activity_streams),
        )
        .route("/api/strava/activities/sync", post(sync_strava_activities))
        .route(
            "/api/strava/activities/{id}/streams/sync",
            post(sync_strava_activity_streams),
        )
        .route(
            "/api/strava/activities/streams/sync",
            post(sync_all_strava_activity_streams),
        )
        .route(
            "/api/activities/{activity_id}/music",
            get(get_activity_music),
        )
        .route_layer(login_required!(AuthBackend))
        .with_state(state.clone().into());

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(from_fn(middleware::handle_errors))
        .layer(cors) // ← Use the new CORS configuration
        .layer(auth_layer)
        .fallback(handler_404);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Run Sous BPM API server starting on port {}", port);
    info!("CORS enabled for: {}", allowed_origin);
    axum::serve(listener, app).await?;

    Ok(())
}
