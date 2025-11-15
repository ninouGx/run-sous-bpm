use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::AuthBackend,
    database::get_user_by_id,
    services::{analytics_service, get_lastfm_tracks_raw},
};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{
    AppState,
    responses::{ActivityMusicResponse, LastFmRangeResponse, LastFmTrackInfo, TrackWithTimestamp},
};

pub async fn get_activity_music(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(activity_id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized"
            })),
        );
    };
    let Ok(activity_id) = Uuid::parse_str(&activity_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Invalid activity ID format"
            })),
        );
    };
    match analytics_service::get_activity_music(&state.db_connection, user.id, activity_id).await {
        Ok(listens_with_tracks) => {
            let tracks: Vec<TrackWithTimestamp> = listens_with_tracks
                .into_iter()
                .map(|(listen, track)| TrackWithTimestamp {
                    played_at: listen.played_at,
                    track_name: track.track_name,
                    artist_name: track.artist_name,
                    album_name: track.album_name,
                    track_id: track.id,
                    listen_id: listen.id,
                })
                .collect();

            let response = ActivityMusicResponse {
                total_tracks: tracks.len(),
                tracks,
            };

            (StatusCode::OK, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": e.to_string()
            })),
        ),
    }
}

/// Query parameters for Last.fm range endpoint
#[derive(Debug, Deserialize, Serialize)]
pub struct LastFmRangeQuery {
    /// Unix timestamp (seconds) for start of range
    pub start: i64,
    /// Unix timestamp (seconds) for end of range
    pub end: i64,
}

/// Debug endpoint to fetch raw Last.fm data for a time range
///
/// This endpoint helps investigate timestamp boundary behavior and
/// understand why certain tracks might not be captured during sync.
///
/// # Query Parameters
/// - `start`: Unix timestamp (seconds) for start of range
/// - `end`: Unix timestamp (seconds) for end of range
///
/// # Example
/// GET /api/music/lastfm/range?start=1730297719&end=1730301319
#[allow(dead_code)]
pub async fn get_lastfm_range(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Query(params): Query<LastFmRangeQuery>,
) -> (StatusCode, Json<Value>) {
    let Some(user) = auth_session.user else {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "error": "Unauthorized"
            })),
        );
    };

    // Get user's Last.fm username
    let user_record = match get_user_by_id(&state.db_connection, user.id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "User not found"
                })),
            );
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("Database error: {}", e)
                })),
            );
        }
    };

    let Some(lastfm_username) = user_record.lastfm_username else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "User does not have a Last.fm username configured"
            })),
        );
    };

    // Fetch raw Last.fm tracks
    match get_lastfm_tracks_raw(&lastfm_username, params.start, params.end).await {
        Ok(tracks) => {
            let track_infos: Vec<LastFmTrackInfo> = tracks
                .into_iter()
                .map(|track| LastFmTrackInfo {
                    track_name: track.name.clone(),
                    artist_name: track.artist.text.clone(),
                    album_name: if track.album.text.is_empty() {
                        None
                    } else {
                        Some(track.album.text.clone())
                    },
                    played_at_timestamp: track.date.as_ref().map(|d| i64::from(d.uts)),
                    played_at_text: track.date.as_ref().map(|d| d.text.clone()),
                })
                .collect();

            let response = LastFmRangeResponse {
                total_tracks: track_infos.len(),
                tracks: track_infos,
                start_timestamp: params.start,
                end_timestamp: params.end,
            };

            (StatusCode::OK, Json(json!(response)))
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": format!("Last.fm API error: {}", e)
            })),
        ),
    }
}
