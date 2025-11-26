use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::{hash_password, AuthBackend, Credentials},
    config::OAuthProvider,
    database::{create_user, get_user_by_email},
    services::is_oauth_provider_connected,
};
use serde_json::{json, Value};
use validator::Validate;

use crate::AppState;

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<Credentials>,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid input",
                "message": e.to_string()
            })),
        );
    }

    // Check if email already exists
    match get_user_by_email(&state.db_connection, payload.email.clone()).await {
        Ok(Some(_)) => {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "Email already registered",
                    "message": "An account with this email already exists"
                })),
            );
        }
        Ok(None) => {
            // Email is available, continue with registration
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Database error",
                    "message": e.to_string()
                })),
            );
        }
    }

    let Ok(hash) = hash_password(&payload.password) else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Password hashing failed"
            })),
        );
    };
    match create_user(&state.db_connection, payload.email, hash).await {
        Ok(user) => (
            StatusCode::CREATED,
            Json(json!({
                "id": user.id,
                "email": user.email,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "User creation failed",
                "message": e.to_string()
            })),
        ),
    }
}

pub async fn login_user(
    mut auth: AuthSession<AuthBackend>,
    Json(payload): Json<Credentials>,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = payload.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid input",
                "message": e.to_string()
            })),
        );
    }
    let user = auth.authenticate(payload).await;
    match user {
        Ok(Some(user)) => {
            if let Err(e) = auth.login(&user).await {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to create session",
                        "message": e.to_string()
                    })),
                );
            }
            (
                StatusCode::OK,
                Json(json!({
                    "message": "Login successful",
                    "user": {
                        "id": user.id,
                        "email": user.email,
                    }
                })),
            )
        }
        Ok(None) => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Invalid credentials"
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Authentication failed",
                "message": e.to_string()
            })),
        ),
    }
}

pub async fn logout_user(mut auth: AuthSession<AuthBackend>) -> (StatusCode, Json<Value>) {
    auth.logout().await.ok();
    (
        StatusCode::OK,
        Json(json!({
            "message": "Logout successful"
        })),
    )
}

pub async fn get_current_user(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
) -> (StatusCode, Json<Value>) {
    match auth_session.user {
        Some(user) => {
            let (is_connected_strava, is_connected_spotify) = {
                (
                    is_oauth_provider_connected(
                        &state.db_connection,
                        user.id,
                        OAuthProvider::Strava,
                    )
                    .await
                    .unwrap_or(false),
                    is_oauth_provider_connected(
                        &state.db_connection,
                        user.id,
                        OAuthProvider::Spotify,
                    )
                    .await
                    .unwrap_or(false),
                )
            };
            (
                StatusCode::OK,
                Json(json!({
                    "id": user.id,
                    "email": user.email,
                    "lastfm_username": user.lastfm_username,
                    "oauth_connections": {
                        "strava": is_connected_strava,
                        "spotify": is_connected_spotify
                    }
                })),
            )
        }
        None => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Not authenticated"
            })),
        ),
    }
}
