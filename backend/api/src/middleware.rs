use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use tracing::{debug, error, warn};

/// Error handling middleware that converts error responses to JSON
/// and logs them with appropriate severity levels
///
/// This middleware:
/// - Lets successful/redirect responses pass through unchanged
/// - Converts 4xx/5xx responses to structured JSON errors
/// - Logs errors with context from the tracing span
#[allow(clippy::too_many_lines)]
pub async fn handle_errors(req: Request<Body>, next: Next) -> Response {
    // Extract path for context in logs (the span will already have this, but useful for manual logs)
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    let response = next.run(req).await;
    let status = response.status();

    // Let successful and redirection responses pass through unchanged
    if status.is_success() || status.is_redirection() {
        return response;
    }

    // From here, we're dealing with errors (4xx or 5xx)
    // The status code is already logged by TraceLayer, so we just add contextual details

    match status {
        StatusCode::BAD_REQUEST => {
            debug!(
                method = %method,
                path = %path,
                "Bad request - invalid input from client"
            );
            (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({
                    "error": "Bad Request",
                    "status": 400,
                    "message": "The request could not be understood or was missing required parameters"
                })
                ),
            ).into_response()
        }
        StatusCode::UNAUTHORIZED => {
            warn!(
                method = %method,
                path = %path,
                "Unauthorized access attempt"
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(
                    json!({
                    "error": "Unauthorized",
                    "status": 401,
                    "message": "Authentication is required and has failed or has not yet been provided"
                })
                ),
            ).into_response()
        }
        StatusCode::FORBIDDEN => {
            warn!(
                method = %method,
                path = %path,
                "Forbidden - insufficient permissions"
            );
            (
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "Forbidden",
                    "status": 403,
                    "message": "You don't have permission to access this resource"
                })),
            )
                .into_response()
        }
        // NOT_FOUND is handled by the fallback handler with custom hint message
        StatusCode::NOT_FOUND => response,
        StatusCode::METHOD_NOT_ALLOWED => {
            warn!(
                method = %method,
                path = %path,
                "Method not allowed for this endpoint"
            );
            (
                StatusCode::METHOD_NOT_ALLOWED,
                Json(json!({
                    "error": "Method Not Allowed",
                    "status": 405,
                    "message": "The requested HTTP method is not allowed for this endpoint"
                })),
            )
                .into_response()
        }
        StatusCode::INTERNAL_SERVER_ERROR => {
            error!(
                method = %method,
                path = %path,
                "Internal server error occurred"
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Internal Server Error",
                    "status": 500,
                    "message": "An unexpected error occurred. The issue has been logged."
                })),
            )
                .into_response()
        }
        StatusCode::BAD_GATEWAY => {
            error!(
                method = %method,
                path = %path,
                "Bad gateway - external service error"
            );
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "error": "Bad Gateway",
                    "status": 502,
                    "message": "Error communicating with external service"
                })),
            )
                .into_response()
        }
        StatusCode::SERVICE_UNAVAILABLE => {
            error!(
                method = %method,
                path = %path,
                "Service temporarily unavailable"
            );
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "error": "Service Unavailable",
                    "status": 503,
                    "message": "The service is temporarily unavailable. Please try again later."
                })),
            )
                .into_response()
        }
        // For other errors, return a generic error response
        _ => {
            warn!(
                method = %method,
                path = %path,
                status = status.as_u16(),
                "Unhandled error status code"
            );
            (
                status,
                Json(json!({
                    "error": status.canonical_reason().unwrap_or("Error"),
                    "status": status.as_u16(),
                    "message": status.canonical_reason().unwrap_or("An error occurred")
                })),
            )
                .into_response()
        }
    }
}
