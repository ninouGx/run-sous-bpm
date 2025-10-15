use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use tracing::info;

pub async fn handle_errors(req: Request<Body>, next: Next) -> Response {
    let response = next.run(req).await;
    let status = response.status();

    info!("Response Status: {}", status);

    // Only intercept error responses (4xx and 5xx)
    // Let successful responses (2xx, 3xx) pass through unchanged
    if status.is_success() || status.is_redirection() {
        return response;
    }

    // Handle specific error cases with custom messages
    match status {
        StatusCode::UNAUTHORIZED => (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized",
                "status": 401,
                "message": "Authentication is required and has failed or has not yet been provided"
            })),
        )
            .into_response(),
        StatusCode::FORBIDDEN => (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "Forbidden",
                "status": 403,
                "message": "You don't have permission to access this resource"
            })),
        )
            .into_response(),
        // NOT_FOUND is handled by the fallback handler with custom hint message
        StatusCode::NOT_FOUND => response,
        StatusCode::METHOD_NOT_ALLOWED => (
            StatusCode::METHOD_NOT_ALLOWED,
            Json(json!({
                "error": "Method Not Allowed",
                "status": 405,
                "message": "The requested HTTP method is not allowed for this endpoint"
            })),
        )
            .into_response(),
        StatusCode::INTERNAL_SERVER_ERROR => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Internal Server Error",
                "status": 500,
                "message": "An unexpected error occurred"
            })),
        )
            .into_response(),
        // For other errors, return a generic error response
        _ => (
            status,
            Json(json!({
                "error": status.canonical_reason().unwrap_or("Error"),
                "status": status.as_u16(),
                "message": status.canonical_reason().unwrap_or("An error occurred")
            })),
        )
            .into_response(),
    }
}
