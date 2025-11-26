use axum::{
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::{json, Value};

pub async fn root() -> Json<Value> {
    Json(json!({
        "message": "Welcome to Run Sous BPM API",
        "version": "0.1.0",
        "endpoints": {
            "health": "/health",
            "auth": {
                "register": "POST /api/auth/register",
                "login": "POST /api/auth/login",
                "logout": "POST /api/auth/logout (requires auth)"
            },
            "oauth": {
                "spotify": "GET /api/oauth/spotify/authorize (requires auth)",
                "strava": "GET /api/oauth/strava/authorize (requires auth)",
                "callback": "GET /api/oauth/callback"
            }
        }
    }))
}

pub async fn handler_404() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": "Not Found",
            "status": 404,
            "message": "The requested resource was not found",
            "hint": "Visit GET / for available endpoints"
        })),
    )
}
