use std::sync::Arc;

use axum::{ Extension, extract::{ Path, Query }, http::StatusCode, response::Json };
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::AuthBackend,
    config::OAuthProvider,
    services::{ OAuthSessionManager, handle_oauth_callback, oauth::start_oauth_flow },
};
use sea_orm::DatabaseConnection;
use serde_json::{ Value, json };

pub async fn oauth_callback(
    Path(provider): Path<String>,
    Extension(session_manager): Extension<Arc<OAuthSessionManager>>,
    auth_session: AuthSession<AuthBackend>
) -> (StatusCode, Json<Value>) {
    let user = match auth_session.user {
        Some(user) => user,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(
                    json!({
                    "error": "Unauthorized",
                    "message": "You must be logged in to connect OAuth providers"
                })
                ),
            );
        }
    };

    match provider.parse::<OAuthProvider>() {
        Ok(provider) => {
            let auth_url = start_oauth_flow(provider, &session_manager, user.id);
            (
                StatusCode::OK,
                Json(json!({
                    "auth_url": auth_url,
                })),
            )
        }
        Err(_) =>
            (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({
                "error": "Invalid provider",
                "message": format!("Provider '{}' is not supported", provider)
            })
                ),
            ),
    }
}

#[derive(serde::Deserialize)]
pub struct OAuthCallbackParams {
    code: String,
    state: String,
}

pub async fn oauth_process_callback(
    params: Query<OAuthCallbackParams>,
    Extension(session_manager): Extension<Arc<OAuthSessionManager>>,
    Extension(db_connection): Extension<DatabaseConnection>
) -> (StatusCode, Json<Value>) {
    let (code, state) = (params.code.clone(), params.state.clone());
    match
        handle_oauth_callback(code.clone(), state.clone(), &session_manager, &db_connection).await
    {
        Ok(_token_response) =>
            (
                StatusCode::OK,
                Json(
                    json!({
                "message": "OAuth process completed successfully",
            })
                ),
            ),
        Err(e) => {
            (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({
                    "error": "Failed to process OAuth callback",
                    "message": e.to_string(),
                })
                ),
            )
        }
    }
}
