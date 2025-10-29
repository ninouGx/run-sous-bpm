use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use axum_login::AuthSession;
use run_sous_bpm_core::{auth::AuthBackend, services::analytics_service};
use sea_orm::prelude::Uuid;
use serde_json::{Value, json};

use crate::{
    AppState,
    responses::{ActivityMusicResponse, TrackWithTimestamp},
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
