use std::sync::Arc;

use axum::{ Json, extract::{ Path, State }, http::StatusCode };
use axum_login::AuthSession;
use run_sous_bpm_core::auth::AuthBackend;
use sea_orm::prelude::Uuid;
use serde_json::{ Value, json };
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
    auth_session: AuthSession<AuthBackend>
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(
                json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })
            ),
        );
    };
    let user_id = user.id;

    match
        run_sous_bpm_core::services::sync_strava_activities(
            user_id,
            &state.strava_client,
            &state.db_connection
        ).await
    {
        Ok(activities) =>
            (
                StatusCode::OK,
                Json(
                    json!(
                { "message": format!("Successfully synced {} activities", activities.len())}
            )
                ),
            ),
        Err(err) =>
            (
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
/// * `id` - The activity's internal UUID
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
    Path(id): Path<String>
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(
                json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })
            ),
        );
    };
    let user_id = user.id;

    let Ok(activity_id) = id.parse::<Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid activity ID format"})));
    };

    info!(user_id = %user_id, activity_id = %activity_id, "Starting sync of Strava activity streams");

    // First get the activity to get its external_id
    match
        run_sous_bpm_core::database::activity_repository::get_activity_by_id(
            &state.db_connection,
            activity_id
        ).await
    {
        Ok(Some(activity)) if activity.user_id == user_id => {
            let external_id = activity.external_id;
            info!(user_id = %user_id, activity_id = %activity_id, external_id = %external_id, "Syncing Strava activity streams");

            match
                run_sous_bpm_core::services::sync_strava_activity_streams(
                    user_id,
                    external_id,
                    &state.strava_client,
                    &state.db_connection
                ).await
            {
                Ok(()) =>
                    (
                        StatusCode::OK,
                        Json(json!({"message": "Successfully synced activity streams"})),
                    ),
                Err(err) =>
                    (
                        StatusCode::BAD_GATEWAY,
                        Json(
                            json!({"error": format!("Failed to sync Strava activity streams: {}", err)})
                        ),
                    ),
            }
        }
        Ok(Some(_)) => { (StatusCode::NOT_FOUND, Json(json!({"error": "Activity not found"}))) }
        Ok(None) => { (StatusCode::NOT_FOUND, Json(json!({"error": "Activity not found"}))) }
        Err(err) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", err)})),
            ),
    }
}

pub async fn sync_all_strava_activity_streams(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(
                json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })
            ),
        );
    };
    let user_id = user.id;

    match
        run_sous_bpm_core::services::sync_all_strava_activity_streams(
            user_id,
            &state.strava_client,
            &state.db_connection
        ).await
    {
        Ok(()) =>
            (StatusCode::OK, Json(json!({"message": "Successfully synced all activity streams"}))),
        Err(err) =>
            (
                StatusCode::BAD_GATEWAY,
                Json(
                    json!({"error": format!("Failed to sync all Strava activity streams: {}", err)})
                ),
            ),
    }
}

/// Retrieves user's Strava activities from the local database
///
/// Returns all activities that have been synced to the database, ordered by start time (most recent first).
///
/// # Returns
///
/// - `200 OK`: JSON array of activities from database
/// - `401 Unauthorized`: User not authenticated
/// - `500 Internal Server Error`: Database query failed
pub async fn get_strava_activities(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(
                json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })
            ),
        );
    };
    let user_id = user.id;

    match
        run_sous_bpm_core::database::activity_repository::get_activities_by_user(
            &state.db_connection,
            user_id
        ).await
    {
        Ok(activities) => (StatusCode::OK, Json(json!(activities))),
        Err(err) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to retrieve activities: {}", err)})),
            ),
    }
}

/// Retrieves detailed stream data for a specific activity from the local database
///
/// Returns time-series data (GPS coordinates, heart rate, cadence, etc.) that has been synced to the database.
///
/// # Arguments
///
/// * `id` - The activity's internal UUID
///
/// # Returns
///
/// - `200 OK`: JSON array containing activity stream data points
/// - `400 Bad Request`: Invalid activity ID format
/// - `401 Unauthorized`: User not authenticated
/// - `404 Not Found`: Activity not found or not owned by user
/// - `500 Internal Server Error`: Database query failed
pub async fn get_strava_activity_streams(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(id): Path<String>
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(
                json!({
                "error": "Unauthorized",
                "message": "You must be logged in to access this resource"
            })
            ),
        );
    };
    let user_id = user.id;

    let Ok(activity_id) = id.parse::<sea_orm::prelude::Uuid>() else {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": "Invalid activity ID format"})));
    };

    // First verify the activity exists and belongs to the user
    match
        run_sous_bpm_core::database::activity_repository::get_activity_by_id(
            &state.db_connection,
            activity_id
        ).await
    {
        Ok(Some(activity)) if activity.user_id == user_id => {
            // Activity found and belongs to user, get streams
            match
                run_sous_bpm_core::database::activity_stream_repository::get_activity_streams(
                    &state.db_connection,
                    activity_id
                ).await
            {
                Ok(streams) => (StatusCode::OK, Json(json!(streams))),
                Err(err) =>
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(
                            json!({"error": format!("Failed to retrieve activity streams: {}", err)})
                        ),
                    ),
            }
        }
        Ok(Some(_)) => {
            // Activity exists but doesn't belong to user
            (StatusCode::NOT_FOUND, Json(json!({"error": "Activity not found"})))
        }
        Ok(None) => {
            // Activity doesn't exist
            (StatusCode::NOT_FOUND, Json(json!({"error": "Activity not found"})))
        }
        Err(err) =>
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Database error: {}", err)})),
            ),
    }
}
