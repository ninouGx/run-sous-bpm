use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::AuthBackend,
    config::OAuthProvider,
    services::{handle_oauth_callback, oauth::start_oauth_flow},
};
use serde_json::{Value, json};

use crate::AppState;

pub async fn oauth_callback(
    Path(provider): Path<String>,
    State(app_state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "message": "You must be logged in to connect OAuth providers"
            })),
        );
    };

    match provider.parse::<OAuthProvider>() {
        Ok(provider) => {
            let auth_url = start_oauth_flow(provider, &app_state.oauth_session_store, user.id);
            (
                StatusCode::OK,
                Json(json!({
                    "auth_url": auth_url,
                })),
            )
        }
        Err(_) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid provider",
                "message": format!("Provider '{}' is not supported", provider)
            })),
        ),
    }
}

#[derive(serde::Deserialize)]
pub struct OAuthCallbackParams {
    code: String,
    state: String,
}

pub async fn oauth_process_callback(
    State(app_state): State<AppState>,
    params: Query<OAuthCallbackParams>,
) -> (StatusCode, Json<Value>) {
    let (code, state) = (params.code.clone(), params.state.clone());
    match handle_oauth_callback(
        code.clone(),
        state.clone(),
        &app_state.oauth_session_store,
        &app_state.db_connection,
    )
    .await
    {
        Ok(_token_response) => (
            StatusCode::OK,
            Json(json!({
                "message": "OAuth process completed successfully",
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Failed to process OAuth callback",
                "message": e.to_string(),
            })),
        ),
    }
}
