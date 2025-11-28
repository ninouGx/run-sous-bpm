use chrono::{DateTime, Utc};
use sea_orm::prelude::Uuid;
use serde::{Deserialize, Serialize};

/// Response for GET /api/activities/{id}/music with GPS segments
#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityMusicResponse {
    pub activity_id: Uuid,
    pub has_gps: bool,
    pub segments: Vec<SegmentResponse>,
    pub stats: SimplificationStats,
}

/// A segment of an activity with GPS points and optional music track
#[derive(Debug, Serialize, Deserialize)]
pub struct SegmentResponse {
    pub index: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<TrackInfo>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub points: Vec<GpsPointResponse>,
}

/// Track information within a segment
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: Uuid,
    pub track_name: String,
    pub artist_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub album_name: Option<String>,
}

/// GPS point with sensor data
#[derive(Debug, Serialize, Deserialize)]
pub struct GpsPointResponse {
    pub time: DateTime<Utc>,
    pub latitude: f64,
    pub longitude: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub altitude: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heart_rate: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cadence: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watts: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub velocity: Option<f32>,
}

/// Statistics about GPS simplification
#[derive(Debug, Serialize, Deserialize)]
pub struct SimplificationStats {
    pub total_segments: usize,
    pub segments_with_music: usize,
    pub segments_without_music: usize,
    pub original_points: usize,
    pub simplified_points: usize,
    pub reduction_ratio: f32,
}
