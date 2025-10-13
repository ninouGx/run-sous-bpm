use std::sync::Arc;

use axum::{ extract::{ Path, Query }, http::StatusCode, response::Json, Extension };
use serde_json::{ json, Value };
use run_sous_bpm_core::{
    config::OAuthProvider,
    services::{ oauth::start_oauth_flow, OAuthSessionManager },
};

pub async fn oauth_callback(
    Path(provider): Path<String>,
    Extension(session_manager): Extension<Arc<OAuthSessionManager>>
) -> (StatusCode, Json<Value>) {
    match provider.parse::<OAuthProvider>() {
        Ok(provider) => {
            let auth_url = start_oauth_flow(provider, &session_manager);
            (StatusCode::OK, Json(json!({
                "auth_url": auth_url,
            })))
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

//GET /api/oauth/callback?code=...&state=...`
// This handler would process the OAuth callback, validate the CSRF token, exchange the code for tokens, and store them securely.
pub async fn oauth_process_callback(
    params: Query<OAuthCallbackParams>,
    Extension(_session_manager): Extension<Arc<OAuthSessionManager>>
) -> (StatusCode, Json<Value>) {
    let (code, state) = (params.code.clone(), params.state.clone());
    // Here you would add the logic to validate the CSRF token (state), exchange the code for tokens,
    (
        StatusCode::OK,
        Json(
            json!({
                "message": "OAuth callback received",
                "code": code,
                "state": state
            })
        ),
    )
}
