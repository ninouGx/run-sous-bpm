mod handlers;
mod middleware;
mod responses;
mod tracing_config;

use axum::extract::MatchedPath;
use axum::http::{HeaderValue, Method, Request, Response};
use axum::{
    middleware::from_fn,
    routing::{get, patch, post},
    Router,
};
use axum_login::{login_required, tower_sessions::MemoryStore, AuthManagerLayerBuilder};
use handlers::{
    get_activity_music, get_current_user, get_strava_activities, get_strava_activity_streams,
    handler_404, health, login_user, logout_user, oauth_callback, oauth_process_callback,
    register_user, root, sync_all_strava_activity_streams, sync_strava_activities,
    sync_strava_activity_streams,
};
use run_sous_bpm_core::crypto::EncryptionService;
use run_sous_bpm_core::{
    auth::AuthBackend, database::establish_db_connection, services::OAuthSessionManager,
};
use run_sous_bpm_integrations::{
    common::{AuthenticatedClient, IntegrationClient},
    strava::StravaApiClient,
};
use sea_orm::DatabaseConnection;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tower_http::classify::ServerErrorsFailureClass;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{
    cookie::{time, SameSite},
    Expiry, SessionManagerLayer,
};
use tracing::{info, info_span, Span};

use crate::handlers::{patch_user, remove_oauth_provider};

#[derive(Clone)]
struct AppState {
    db_connection: DatabaseConnection,
    oauth_session_store: Arc<OAuthSessionManager>,
    strava_client: Arc<StravaApiClient>,
    encryption_service: Arc<EncryptionService>,
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_config::init_tracing();

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

    let encryption_key_path =
        std::env::var("ENCRYPTION_KEY_FILE").expect("ENCRYPTION_KEY_FILE must be set in .env");
    let encryption_service = Arc::new(
        EncryptionService::from_file(Path::new(&encryption_key_path))
            .expect("Failed to initialize EncryptionService from key file"),
    );
    info!("Encryption service initialized successfully");

    let state = AppState {
        db_connection: db_connection.clone(),
        oauth_session_store: oauth_session_store.clone(),
        strava_client,
        encryption_service,
    };

    let session_store = MemoryStore::default();

    // Session configuration with security best practices
    // - HttpOnly: prevents JavaScript access to cookies (default in tower_sessions)
    // - Secure: only send cookie over HTTPS (configurable via COOKIE_SECURE env var)
    // - SameSite::Strict: only send cookie for same-site requests (prevents CSRF)
    let cookie_secure = std::env::var("COOKIE_SECURE")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(false); // Default to false for local development

    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("run_sous_bpm_session")
        .with_secure(cookie_secure)
        .with_same_site(SameSite::Strict) // Changed from Lax to Strict for better security
        .with_http_only(true) // Explicitly set HttpOnly (prevents XSS attacks)
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));
    let auth_backend = AuthBackend::new(db_connection.clone());
    let auth_layer = AuthManagerLayerBuilder::new(auth_backend, session_layer).build();

    let oauth_callback_route =
        std::env::var("REDIRECT_ENDPOINT").unwrap_or_else(|_| "/api/oauth/callback".to_string());

    let allowed_origin = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");

    let cors = CorsLayer::new()
        .allow_origin(
            allowed_origin
                .parse::<HeaderValue>()
                .expect("Invalid CORS origin"),
        )
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

    // Create trace layer with comprehensive request/response logging
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<axum::body::Body>| {
            let matched_path = request
                .extensions()
                .get::<MatchedPath>()
                .map_or("unknown", MatchedPath::as_str);

            let user_id = request
                .extensions()
                .get::<axum_login::AuthSession<AuthBackend>>()
                .and_then(|auth| auth.user.as_ref())
                .map_or_else(|| "anonymous".to_string(), |user| user.id.to_string());

            info_span!(
                "http_request",
                method = %request.method(),
                matched_path,
                uri = %request.uri(),
                version = ?request.version(),
                user_id,
                status = tracing::field::Empty,
                latency_ms = tracing::field::Empty,
            )
        })
        .on_request(|_request: &Request<axum::body::Body>, _span: &Span| {
            tracing::debug!("Request started");
        })
        .on_response(
            |response: &Response<axum::body::Body>, latency: Duration, span: &Span| {
                let status = response.status();
                let latency_ms = latency.as_millis();

                span.record("status", status.as_u16());
                span.record("latency_ms", latency_ms);

                match status.as_u16() {
                    200..=299 => {
                        tracing::info!(
                            status = status.as_u16(),
                            latency_ms,
                            "Request completed successfully"
                        );
                    }
                    300..=399 => {
                        tracing::info!(status = status.as_u16(), latency_ms, "Request redirected");
                    }
                    400..=499 => {
                        tracing::warn!(status = status.as_u16(), latency_ms, "Client error");
                    }
                    500..=599 => {
                        tracing::error!(status = status.as_u16(), latency_ms, "Server error");
                    }
                    _ => {
                        tracing::info!(status = status.as_u16(), latency_ms, "Request completed");
                    }
                }
            },
        )
        .on_failure(
            |error: ServerErrorsFailureClass, latency: Duration, span: &Span| {
                let latency_ms = latency.as_millis();
                span.record("latency_ms", latency_ms);

                tracing::error!(error = ?error, latency_ms, "Request failed");
            },
        );

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
        .layer(from_fn(middleware::handle_errors))
        .layer(trace_layer)
        .layer(cors)
        .layer(auth_layer)
        .fallback(handler_404);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Run Sous BPM API server starting on port {}", port);
    info!("CORS enabled for: {}", allowed_origin);

    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C signal handler");
        info!("Received Ctrl+C signal, initiating graceful shutdown...");
    };

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal)
        .await?;

    info!("Server shutdown complete");

    Ok(())
}
