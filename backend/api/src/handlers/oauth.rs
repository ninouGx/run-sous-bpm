use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Json, Redirect},
};
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::AuthBackend,
    config::OAuthProvider,
    services::{handle_oauth_callback, oauth::start_oauth_flow},
};
use serde_json::{json, Value};
use tracing::info;

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
) -> Redirect {
    let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");

    let (code, state) = (params.code.clone(), params.state.clone());
    match handle_oauth_callback(
        code.clone(),
        state.clone(),
        &app_state.oauth_session_store,
        &app_state.db_connection,
        &app_state.encryption_service,
    )
    .await
    {
        Ok((_token_response, provider)) => {
            let provider_str = provider.to_string().to_lowercase();
            let redirect_url =
                format!("{frontend_url}/oauth/callback?status=success&provider={provider_str}");
            info!(
                "OAuth callback successful for provider: {provider_str}, redirecting to {redirect_url}"
            );
            Redirect::to(&redirect_url)
        }
        Err(e) => {
            let error_string = e.to_string();
            let error_message = urlencoding::encode(&error_string);
            let redirect_url =
                format!("{frontend_url}/oauth/callback?status=error&error={error_message}");
            info!("OAuth callback failed: {error_string}, redirecting to {redirect_url}");
            Redirect::to(&redirect_url)
        }
    }
}

pub async fn remove_oauth_provider(
    State(app_state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(provider): Path<String>,
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "message": "You must be logged in to disconnect OAuth providers"
            })),
        );
    };

    info!(user_id = %user.id, provider = %provider, "Removing OAuth provider connection");

    match provider.parse::<OAuthProvider>() {
        Ok(provider) => {
            match run_sous_bpm_core::database::repositories::delete_oauth_token(
                &app_state.db_connection,
                user.id,
                provider,
            )
            .await
            {
                Ok(()) => (
                    StatusCode::OK,
                    Json(json!({
                        "message": format!("Successfully disconnected {provider}"),
                    })),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to disconnect provider",
                        "message": e.to_string(),
                    })),
                ),
            }
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
