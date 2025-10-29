use chrono::DateTime;
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

/// Response for GET /api/activities/{id}/music
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityMusicResponse {
    pub tracks: Vec<TrackWithTimestamp>,

    pub total_tracks: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackWithTimestamp {
    pub played_at: DateTime<chrono::FixedOffset>,

    pub track_name: String,

    pub artist_name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,

    /// Track UUID (for future enrichment, links, etc.)
    pub track_id: Uuid,

    /// Listen UUID (if needed for updates/deletes)
    pub listen_id: Uuid,
}
