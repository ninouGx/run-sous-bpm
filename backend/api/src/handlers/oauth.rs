use axum::{ extract::Path, http::StatusCode, response::Json };
use serde_json::{ json, Value };
use run_sous_bpm_core::models::OAuthProvider;

pub async fn oauth_callback(Path(provider): Path<String>) -> (StatusCode, Json<Value>) {
    match provider.parse::<OAuthProvider>() {
        Ok(_) => {
            (
                StatusCode::OK,
                Json(
                    json!({
                "message": "OAuth authorization URL generated",
                "provider": provider
            })
                ),
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
