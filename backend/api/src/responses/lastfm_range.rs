use serde::{Deserialize, Serialize};

/// Response for GET /api/music/lastfm/range
///
/// Returns raw Last.fm data for debugging timestamp boundaries
#[derive(Debug, Serialize, Deserialize)]
pub struct LastFmRangeResponse {
    /// Tracks returned from Last.fm API
    pub tracks: Vec<LastFmTrackInfo>,

    /// Total number of tracks returned
    pub total_tracks: usize,

    /// The start timestamp used in the query (for verification)
    pub start_timestamp: i64,

    /// The end timestamp used in the query (for verification)
    pub end_timestamp: i64,
}

/// Simplified track information from Last.fm
#[derive(Debug, Serialize, Deserialize)]
pub struct LastFmTrackInfo {
    /// Track name
    pub track_name: String,

    /// Artist name
    pub artist_name: String,

    /// Album name (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,

    /// Unix timestamp when track was played (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub played_at_timestamp: Option<i64>,

    /// Human-readable date string (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub played_at_text: Option<String>,
}
