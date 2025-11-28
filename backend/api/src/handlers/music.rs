use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_login::AuthSession;
use run_sous_bpm_core::{
    auth::AuthBackend,
    database::get_user_by_id,
    services::{analytics_service, get_lastfm_tracks_raw},
};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    responses::{
        ActivityMusicResponse, GpsPointResponse, LastFmRangeResponse, LastFmTrackInfo,
        SegmentResponse, SimplificationStats, TrackInfo,
    },
    AppState,
};

/// Query parameters for activity music endpoint
#[derive(Debug, Deserialize)]
pub struct SimplificationQuery {
    /// Whether to apply GPS simplification
    pub simplify: Option<bool>,
    /// Simplification tolerance in meters (default: 10.0)
    pub tolerance: Option<f64>,
}

pub async fn get_activity_music(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<AuthBackend>,
    Path(activity_id): Path<String>,
    Query(params): Query<SimplificationQuery>,
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
    match analytics_service::get_activity_music(
        &state.db_connection,
        user.id,
        activity_id,
        params.simplify.unwrap_or(true),
        params.tolerance,
    )
    .await
    {
        Ok((segments, simplification_stats)) => {
            // Convert service layer Segment to API SegmentResponse
            let segment_responses: Vec<SegmentResponse> = segments
                .into_iter()
                .map(|segment| {
                    let track = segment.track.map(|t| TrackInfo {
                        id: t.id,
                        track_name: t.track_name,
                        artist_name: t.artist_name,
                        album_name: t.album_name,
                    });

                    let points: Vec<GpsPointResponse> = segment
                        .points
                        .into_iter()
                        .filter_map(|p| match (p.latitude, p.longitude) {
                            (Some(lat), Some(lng)) => Some(GpsPointResponse {
                                time: p.time.with_timezone(&chrono::Utc),
                                latitude: lat,
                                longitude: lng,
                                altitude: p.altitude,
                                heart_rate: p.heart_rate,
                                cadence: p.cadence,
                                watts: p.watts,
                                velocity: p.velocity,
                            }),
                            _ => None,
                        })
                        .collect();

                    SegmentResponse {
                        index: segment.index,
                        track,
                        start_time: segment.start_time,
                        end_time: segment.end_time,
                        points,
                    }
                })
                .collect();

            let has_gps = segment_responses.iter().any(|s| !s.points.is_empty());

            let response = ActivityMusicResponse {
                activity_id,
                has_gps,
                segments: segment_responses,
                stats: SimplificationStats {
                    total_segments: simplification_stats.total_segments,
                    segments_with_music: simplification_stats.segments_with_music,
                    segments_without_music: simplification_stats.segments_without_music,
                    original_points: simplification_stats.original_points,
                    simplified_points: simplification_stats.simplified_points,
                    reduction_ratio: simplification_stats.reduction_ratio,
                },
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
