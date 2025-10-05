use axum::{ http::StatusCode, response::Json };
use serde_json::{ json, Value };

pub async fn health() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(
            json!({
            "status": "healthy",
            "service": "run-sous-bpm-api",
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
        ),
    )
}
