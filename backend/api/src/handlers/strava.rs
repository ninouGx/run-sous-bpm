use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_login::AuthSession;
use run_sous_bpm_core::{auth::AuthBackend, config::OAuthProvider, services::get_valid_token};
use run_sous_bpm_integrations::strava::{
    StravaActivityStreamsParams, client::StravaActivitiesParams,
};
use serde_json::{Value, json};
use tracing::info;

use crate::AppState;

/// Syncs user's Strava activities from the Strava API to the local database
///
/// Fetches all activities for the authenticated user from Strava and stores them locally.
/// Updates existing activities if they already exist (based on `external_id`).
///
/// # Returns
///
/// - `200 OK`: Successfully synced activities with count
/// - `401 Unauthorized`: User not authenticated
/// - `502 Bad Gateway`: Failed to retrieve OAuth token or Strava API error
pub async fn sync_strava_activities(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
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
    let user_id = user.id;

    match run_sous_bpm_core::services::sync_strava_activities(
        user_id,
        &state.strava_client,
        &state.db_connection,
    )
    .await
    {
        Ok(activities) => (
            StatusCode::OK,
            Json(json!(
                { "message": format!("Successfully synced {} activities", activities.len())}
            )),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Failed to sync Strava activities: {}", err)})),
        ),
    }
}

/// Syncs detailed activity stream data for a specific Strava activity
///
/// Fetches time-series data (GPS coordinates, heart rate, cadence, etc.) for a specific activity
/// and stores it in the `TimescaleDB` hypertable for efficient time-series queries.
///
/// # Arguments
///
/// * `id` - The Strava activity external ID
///
/// # Returns
///
/// - `200 OK`: Successfully synced activity streams
/// - `400 Bad Request`: Invalid activity ID format
/// - `401 Unauthorized`: User not authenticated
/// - `502 Bad Gateway`: Failed to retrieve activity or Strava API error
pub async fn sync_strava_activity_streams(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(id): Path<String>,
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
    let user_id = user.id;
    info!(user_id = %user_id, activity_external_id = %id, "Starting sync of Strava activity streams");
    let Ok(external_id) = id.parse::<i64>() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid activity ID"})),
        );
    };

    info!(user_id = %user_id, activity_external_id = %external_id, "Syncing Strava activity streams");

    match run_sous_bpm_core::services::sync_strava_activity_streams(
        user_id,
        external_id,
        &state.strava_client,
        &state.db_connection,
    )
    .await
    {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({"message": "Successfully synced activity streams"})),
        ),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Failed to sync Strava activity streams: {}", err)})),
        ),
    }
}

/// Retrieves user's Strava activities directly from the Strava API
///
/// Fetches activities from Strava API without storing them. Useful for browsing
/// activities before deciding to sync them to the local database.
///
/// # Query Parameters
///
/// * `before` - Return activities before this Unix timestamp
/// * `after` - Return activities after this Unix timestamp
/// * `per_page` - Number of activities per page (default 30)
/// * `page` - Page number
///
/// # Returns
///
/// - `200 OK`: JSON array of Strava activities
/// - `401 Unauthorized`: User not authenticated or invalid OAuth token
/// - `502 Bad Gateway`: Strava API request failed
pub async fn get_strava_activities(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Query(params): Query<StravaActivitiesParams>,
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
    let user_id = user.id;

    // Get valid OAuth token for the user
    let access_token =
        match get_valid_token(&state.db_connection, user_id, OAuthProvider::Strava).await {
            Ok(token) => token,
            Err(err) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": format!("Failed to get Strava access token: {}", err)})),
                );
            }
        };

    match state
        .strava_client
        .get_athlete_activities(&access_token, Some(params))
        .await
    {
        Ok(activities) => (StatusCode::OK, Json(json!(activities))),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Strava API request failed: {}", err)})),
        ),
    }
}

/// Retrieves detailed stream data for a specific Strava activity
///
/// Fetches time-series data directly from the Strava API without storing it.
/// Includes GPS coordinates, heart rate, cadence, power, velocity, and other sensor data.
///
/// # Arguments
///
/// * `id` - The Strava activity external ID
///
/// # Query Parameters
///
/// * `keys` - Comma-separated list of stream types to retrieve
///
/// # Returns
///
/// - `200 OK`: JSON object containing requested activity streams
/// - `400 Bad Request`: Invalid activity ID format
/// - `401 Unauthorized`: User not authenticated or invalid OAuth token
/// - `502 Bad Gateway`: Strava API request failed
pub async fn get_strava_activity_streams(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(id): Path<String>,
    Query(params): Query<StravaActivityStreamsParams>,
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
    let user_id = user.id;
    let Ok(external_id) = id.parse::<i64>() else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Invalid activity ID"})),
        );
    };

    // Get valid OAuth token for the user
    let access_token =
        match get_valid_token(&state.db_connection, user_id, OAuthProvider::Strava).await {
            Ok(token) => token,
            Err(err) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": format!("Failed to get Strava access token: {}", err)})),
                );
            }
        };

    match state
        .strava_client
        .get_activity_streams(&access_token, external_id, params)
        .await
    {
        Ok(streams) => (StatusCode::OK, Json(json!(streams))),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": format!("Strava API request failed: {}", err)})),
        ),
    }
}
