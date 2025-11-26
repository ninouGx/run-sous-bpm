use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use axum_login::AuthSession;
use run_sous_bpm_core::auth::AuthBackend;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub lastfm_username: Option<String>,
}

pub async fn patch_user(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Json(payload): Json<UpdateUserRequest>,
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })),
        );
    };

    let Some(lastfm_username) = payload.lastfm_username else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Bad Request",
                "message": "lastfm_username is required"
            })),
        );
    };

    match run_sous_bpm_core::services::user_service::update_valid_user_lastfm_username(
        user.id,
        lastfm_username,
        &state.db_connection,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "message": "Last.fm username updated successfully"
            })),
        ),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("Invalid Last.fm username") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };

            (
                status,
                Json(json!({
                    "error": status.canonical_reason().unwrap_or("Error"),
                    "message": error_msg
                })),
            )
        }
    }
}
