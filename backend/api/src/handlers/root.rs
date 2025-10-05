use axum::{ http::StatusCode, response::Json };
use serde_json::{ json, Value };

pub async fn root() -> Json<Value> {
    Json(json!({
        "message": "Welcome to Run Sous BPM API",
        "version": "0.1.0"
    }))
}

pub async fn handler_404() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(
            json!({
            "error": "Not Found",
            "message": "The requested resource was not found"
        })
        ),
    )
}
