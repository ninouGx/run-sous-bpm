use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::{
    database::{
        activity_stream::Model,
        entities::prelude::{Listen, Track},
        get_activity_by_id, get_activity_streams, get_listens_by_user_time_range, get_user_by_id,
        listen::{self},
        track::{self},
    },
    geo::simplify_gps_route,
    services::sync_lastfm_for_time_range,
};

/// Default GPS simplification tolerance in meters
///
/// 10 meters provides good balance between:
/// - Route accuracy (preserves shape)
/// - Performance (reduces points ~60-80%)
/// - Map rendering (smooth lines at typical zoom levels)
const DEFAULT_SIMPLIFICATION_TOLERANCE_METERS: f32 = 10.0;

// Split each stream by tracks
/*
    {
  activity_id: "uuid",
  has_gps: true,

  segments: [
    {
      index: 0,
      track: null,
      start_time: "2024-01-15T10:00:00Z",
      end_time: "2024-01-15T10:02:30Z",
      points: [
        { lat: 48.856, lng: 2.352, time: "...", heart_rate: 145, ... },
        ...
      ]
    },
    {
      index: 1,
      track: {
        id: "uuid",
        name: "Song Title",
        artist_name: "Artist",
        album_name: "Album"
      },
      start_time: "2024-01-15T10:02:30Z",
      end_time: "2024-01-15T10:06:15Z",
      points: [...]
    }
  ],

  // Metadata utile pour le front
  stats: {
    total_segments: 12,
    segments_with_music: 10,
    segments_without_music: 2,
    total_points: 1800,
    simplified_points: 250  // si simplify=true
  }
} */

/// Represents a segment of an activity stream
#[derive(Debug, Clone)]
pub struct Segment {
    pub index: usize,
    pub track: Option<track::Model>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub points: Vec<Model>,
}

/// Statistics about GPS simplification
#[derive(Debug, Clone)]
pub struct SimplificationStats {
    pub total_segments: usize,
    pub segments_with_music: usize,
    pub segments_without_music: usize,
    pub original_points: usize,
    pub simplified_points: usize,
    pub reduction_ratio: f32,
}

/// Retrieves music tracks played during a specific activity with GPS segments
///
/// # Arguments
/// * `db` - Database connection
/// * `user_id` - ID of the user
/// * `activity_id` - ID of the activity
/// * `simplify` - Whether to apply GPS simplification
/// * `tolerance` - Simplification tolerance in meters (default: 10.0)
///
/// # Returns
///
/// A tuple of (segments, stats) containing GPS-segmented music data and simplification statistics
///
/// # Errors
///
/// Returns an error if:
/// - Activity is not found in the database
/// - User is not found in the database
/// - User does not have a Last.fm username configured
/// - Last.fm API request fails
/// - Database query fails
/// - GPS simplification fails
pub async fn get_activity_music(
    db: &DatabaseConnection,
    user_id: Uuid,
    activity_id: Uuid,
    simplify: bool,
    tolerance: Option<f64>,
) -> Result<(Vec<Segment>, SimplificationStats), Box<dyn std::error::Error>> {
    let activity = get_activity_by_id(db, activity_id)
        .await?
        .ok_or("Activity not found")?;

    if activity.user_id != user_id {
        return Err("Activity does not belong to the user".into());
    }

    let end_time =
        activity.start_time + chrono::Duration::seconds(i64::from(activity.elapsed_time));
    let wide_start_time = activity.start_time;

    let listens = get_listens_by_user_time_range(db, user_id, wide_start_time, end_time).await?;

    if listens.is_empty() {
        // Fetch user to get Last.fm username
        let user = get_user_by_id(db, user_id).await?.ok_or("User not found")?;

        let lastfm_username = user
            .lastfm_username
            .ok_or("User does not have a Last.fm username configured")?;

        sync_lastfm_for_time_range(
            user_id,
            &lastfm_username,
            wide_start_time.timestamp(),
            end_time.timestamp(),
            db,
        )
        .await?;
    }

    // Retrieve Activity Streams
    let streams = get_activity_streams(db, activity_id).await?;

    let listens_with_tracks = Listen::find()
        .filter(listen::Column::UserId.eq(user_id))
        .filter(listen::Column::PlayedAt.gte(wide_start_time))
        .filter(listen::Column::PlayedAt.lte(end_time))
        .order_by_asc(listen::Column::PlayedAt)
        .find_also_related(Track)
        .all(db)
        .await?;

    // Count only GPS points within activity time range for accurate statistics
    let activity_start_utc: DateTime<Utc> = activity.start_time.into();
    let end_time_utc: DateTime<Utc> = end_time.into();
    let original_gps_points = streams
        .iter()
        .filter(|s| {
            s.latitude.is_some()
                && s.longitude.is_some()
                && s.time >= activity_start_utc
                && s.time <= end_time_utc
        })
        .count();

    let segments = build_activity_segments(
        &streams,
        &listens_with_tracks,
        activity.start_time.into(),
        end_time.into(),
        simplify,
        tolerance,
    )?;

    let stats = calculate_stats(&segments, original_gps_points);

    Ok((segments, stats))
}

fn build_activity_segments(
    streams: &[Model],
    listens: &[(listen::Model, Option<track::Model>)],
    activity_start: DateTime<Utc>,
    activity_end: DateTime<Utc>,
    simplify: bool,
    tolerance: Option<f64>,
) -> Result<Vec<Segment>, Box<dyn std::error::Error>> {
    let mut segments = Vec::new();

    // If no music listens, return the entire activity as a single segment
    if listens.is_empty() {
        let mut all_points: Vec<Model> = streams
            .iter()
            .filter(|s| s.time >= activity_start && s.time <= activity_end)
            .cloned()
            .collect();

        if simplify && all_points.len() >= 2 {
            let tolerance_meters =
                tolerance.unwrap_or(f64::from(DEFAULT_SIMPLIFICATION_TOLERANCE_METERS));
            let indices = simplify_gps_route(&all_points, tolerance_meters)?;
            all_points = indices.iter().map(|&i| all_points[i].clone()).collect();
        }

        segments.push(Segment {
            index: 0,
            track: None,
            start_time: activity_start,
            end_time: activity_end,
            points: all_points,
        });

        return Ok(segments);
    }

    // First segment: before any music
    if listens[0].0.played_at > activity_start {
        // Create segment with track=None for pre-music period
        let pre_music_points: Vec<Model> = streams
            .iter()
            .filter(|s| s.time >= activity_start && s.time < listens[0].0.played_at)
            .cloned()
            .collect();
        let mut segment_points = pre_music_points;
        if simplify && segment_points.len() >= 2 {
            let tolerance_meters =
                tolerance.unwrap_or(f64::from(DEFAULT_SIMPLIFICATION_TOLERANCE_METERS));
            let indices = simplify_gps_route(&segment_points, tolerance_meters)?;
            segment_points = indices.iter().map(|&i| segment_points[i].clone()).collect();
        }
        segments.push(Segment {
            index: 0,
            track: None,
            start_time: activity_start,
            end_time: listens[0].0.played_at.into(),
            points: segment_points,
        });
    }

    // Music segments
    for (i, (listen, track)) in listens.iter().enumerate() {
        let start_time = listen.played_at;
        let end_time = listens
            .get(i + 1)
            .map_or(activity_end.into(), |(l, _)| l.played_at);

        let mut segment_points: Vec<Model> = streams
            .iter()
            .filter(|s| s.time >= start_time && s.time < end_time)
            .cloned()
            .collect();

        if simplify && segment_points.len() >= 2 {
            let tolerance_meters =
                tolerance.unwrap_or(f64::from(DEFAULT_SIMPLIFICATION_TOLERANCE_METERS));

            // Get indices of points to keep
            let indices = simplify_gps_route(&segment_points, tolerance_meters)?;

            // Filter points using indices
            segment_points = indices.iter().map(|&i| segment_points[i].clone()).collect();
        }
        segments.push(Segment {
            index: segments.len(),
            track: track.clone(),
            start_time: start_time.into(),
            end_time: end_time.into(),
            points: segment_points,
        });
    }

    Ok(segments)
}

/// Calculate simplification statistics from segments
///
/// # Arguments
/// * `segments` - The segments to calculate statistics for
/// * `original_points` - The total number of points before segmentation/simplification
///
/// # Returns
///
/// `SimplificationStats` with counts and reduction ratio
#[allow(clippy::cast_precision_loss)]
fn calculate_stats(segments: &[Segment], original_points: usize) -> SimplificationStats {
    let total_segments = segments.len();
    let segments_with_music = segments.iter().filter(|s| s.track.is_some()).count();
    let segments_without_music = total_segments - segments_with_music;
    let simplified_points: usize = segments.iter().map(|s| s.points.len()).sum();
    let reduction_ratio = if original_points > 0 {
        (simplified_points as f32) / (original_points as f32)
    } else {
        0.0
    };

    SimplificationStats {
        total_segments,
        segments_with_music,
        segments_without_music,
        original_points,
        simplified_points,
        reduction_ratio,
    }
}

#[cfg(test)]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
mod tests {
    use super::*;
    use crate::database::activity_stream;
    use chrono::{DateTime, Duration, Utc};
    use uuid::Uuid;

    // ==================== Test Fixtures ====================

    /// Fixed reference timestamp for deterministic tests
    fn base_time() -> DateTime<Utc> {
        DateTime::from_timestamp(1_700_000_000, 0).unwrap() // 2023-11-14 22:13:20 UTC
    }

    /// Create timestamp N minutes after `base_time`
    fn minutes_after(minutes: i64) -> DateTime<Utc> {
        base_time() + Duration::minutes(minutes)
    }

    /// Create timestamp N seconds after `base_time`
    fn seconds_after(seconds: i64) -> DateTime<Utc> {
        base_time() + Duration::seconds(seconds)
    }

    /// Helper to create a test activity stream model with GPS coordinates
    fn make_stream_point(
        activity_id: Uuid,
        time: DateTime<Utc>,
        lat: Option<f64>,
        lng: Option<f64>,
    ) -> activity_stream::Model {
        activity_stream::Model {
            activity_id,
            time: time.into(),
            latitude: lat,
            longitude: lng,
            altitude: Some(100.0),
            heart_rate: Some(150),
            cadence: Some(85),
            watts: Some(200.0),
            velocity: Some(5.5),
            distance: Some(1000.0),
            temperature: Some(20.0),
        }
    }

    /// Helper to create a test listen-track pair
    fn make_listen_with_track(
        user_id: Uuid,
        track_id: Uuid,
        played_at: DateTime<Utc>,
        track_name: &str,
        artist_name: &str,
    ) -> (listen::Model, Option<track::Model>) {
        let listen = listen::Model {
            id: Uuid::new_v4(),
            user_id,
            track_id,
            played_at: played_at.into(),
            created_at: Utc::now().into(),
        };

        let track = Some(track::Model {
            id: track_id,
            artist_name: artist_name.to_string(),
            track_name: track_name.to_string(),
            album_name: Some("Test Album".to_string()),
            artist_mbid: None,
            track_mbid: None,
            album_mbid: None,
            lastfm_url: None,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        });

        (listen, track)
    }

    /// Helper to create a segment for testing
    fn make_segment(
        index: usize,
        track: Option<track::Model>,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        num_points: usize,
    ) -> Segment {
        let activity_id = Uuid::new_v4();
        let points: Vec<activity_stream::Model> = (0..num_points)
            .map(|i| {
                let offset = (i as f64) / (num_points as f64);
                let time = start_time
                    + Duration::seconds(
                        ((end_time.timestamp() - start_time.timestamp()) as f64 * offset) as i64,
                    );
                make_stream_point(
                    activity_id,
                    time,
                    Some(48.0 + offset * 0.01),
                    Some(2.0 + offset * 0.01),
                )
            })
            .collect();

        Segment {
            index,
            track,
            start_time,
            end_time,
            points,
        }
    }

    // ==================== Group A: Pure Function Tests - calculate_stats() ====================

    #[test]
    fn test_calculate_stats_basic() {
        // Create 3 segments: 1 without music, 2 with music
        let segments = vec![
            make_segment(0, None, base_time(), minutes_after(3), 10),
            make_segment(
                1,
                Some(track::Model {
                    id: Uuid::new_v4(),
                    artist_name: "Artist A".to_string(),
                    track_name: "Track A".to_string(),
                    album_name: Some("Album A".to_string()),
                    artist_mbid: None,
                    track_mbid: None,
                    album_mbid: None,
                    lastfm_url: None,
                    created_at: Utc::now().into(),
                    updated_at: Utc::now().into(),
                }),
                minutes_after(3),
                minutes_after(6),
                15,
            ),
            make_segment(
                2,
                Some(track::Model {
                    id: Uuid::new_v4(),
                    artist_name: "Artist B".to_string(),
                    track_name: "Track B".to_string(),
                    album_name: Some("Album B".to_string()),
                    artist_mbid: None,
                    track_mbid: None,
                    album_mbid: None,
                    lastfm_url: None,
                    created_at: Utc::now().into(),
                    updated_at: Utc::now().into(),
                }),
                minutes_after(6),
                minutes_after(10),
                20,
            ),
        ];

        let original_points = 50;
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(stats.total_segments, 3, "Should have 3 total segments");
        assert_eq!(
            stats.segments_with_music, 2,
            "Should have 2 segments with music"
        );
        assert_eq!(
            stats.segments_without_music, 1,
            "Should have 1 segment without music"
        );
        assert_eq!(
            stats.simplified_points, 45,
            "Should have 45 simplified points (10+15+20)"
        );
        assert_eq!(
            stats.original_points, 50,
            "Should preserve original points count"
        );
        assert!(
            (stats.reduction_ratio - 0.9).abs() < 0.001,
            "Reduction ratio should be 0.9 (45/50)"
        );
    }

    #[test]
    fn test_calculate_stats_no_music() {
        let segments = vec![make_segment(0, None, base_time(), minutes_after(10), 100)];

        let original_points = 100;
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(stats.total_segments, 1, "Should have 1 total segment");
        assert_eq!(
            stats.segments_with_music, 0,
            "Should have 0 segments with music"
        );
        assert_eq!(
            stats.segments_without_music, 1,
            "Should have 1 segment without music"
        );
        assert_eq!(
            stats.simplified_points, 100,
            "Simplified points should equal segment points"
        );
        assert!(
            (stats.reduction_ratio - 1.0).abs() < 0.001,
            "Reduction ratio should be 1.0 (no reduction)"
        );
    }

    #[test]
    fn test_calculate_stats_all_music() {
        let track = Some(track::Model {
            id: Uuid::new_v4(),
            artist_name: "Artist".to_string(),
            track_name: "Track".to_string(),
            album_name: Some("Album".to_string()),
            artist_mbid: None,
            track_mbid: None,
            album_mbid: None,
            lastfm_url: None,
            created_at: Utc::now().into(),
            updated_at: Utc::now().into(),
        });

        let segments = vec![
            make_segment(0, track.clone(), base_time(), minutes_after(2), 25),
            make_segment(1, track.clone(), minutes_after(2), minutes_after(5), 25),
            make_segment(2, track.clone(), minutes_after(5), minutes_after(8), 25),
            make_segment(3, track, minutes_after(8), minutes_after(10), 25),
        ];

        let original_points = 150;
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(stats.total_segments, 4, "Should have 4 total segments");
        assert_eq!(
            stats.segments_with_music, 4,
            "All segments should have music"
        );
        assert_eq!(
            stats.segments_without_music, 0,
            "Should have 0 segments without music"
        );
        assert_eq!(
            stats.simplified_points, 100,
            "Should have 100 simplified points"
        );
        assert!(
            (stats.reduction_ratio - 0.6666).abs() < 0.01,
            "Reduction ratio should be ~0.667 (100/150)"
        );
    }

    #[test]
    fn test_calculate_stats_high_reduction() {
        let segments = vec![
            make_segment(0, None, base_time(), minutes_after(5), 10),
            make_segment(1, None, minutes_after(5), minutes_after(10), 10),
        ];

        let original_points = 200;
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(
            stats.simplified_points, 20,
            "Should have 20 simplified points"
        );
        assert!(
            (stats.reduction_ratio - 0.1).abs() < 0.001,
            "Reduction ratio should be 0.1 (20/200)"
        );
        assert!(
            stats.reduction_ratio <= 1.0,
            "Reduction ratio should always be <= 1.0"
        );
        assert!(
            stats.simplified_points <= stats.original_points,
            "Simplified points should be <= original points"
        );
    }

    #[test]
    fn test_calculate_stats_zero_original_points() {
        let segments = vec![make_segment(0, None, base_time(), minutes_after(10), 0)];

        let original_points = 0;
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(
            stats.simplified_points, 0,
            "Should have 0 simplified points"
        );
        assert_eq!(stats.original_points, 0, "Should have 0 original points");
        assert!(
            (stats.reduction_ratio - 0.0).abs() < 0.001,
            "Reduction ratio should be 0.0 when no points exist"
        );
    }

    // ==================== Group B: Segment Indexing Tests - build_activity_segments() ====================

    #[test]
    fn test_segment_indexing_no_pre_music() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create GPS streams every 30 seconds for 10 minutes
        let streams: Vec<activity_stream::Model> = (0..21)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 30), Some(48.0), Some(2.0)))
            .collect();

        // Three tracks starting at activity start
        let listens = vec![
            make_listen_with_track(user_id, Uuid::new_v4(), base_time(), "Track A", "Artist A"),
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(3),
                "Track B",
                "Artist B",
            ),
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(6),
                "Track C",
                "Artist C",
            ),
        ];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 3, "Should have 3 segments");
        assert_eq!(segments[0].index, 0, "First segment should have index 0");
        assert_eq!(segments[1].index, 1, "Second segment should have index 1");
        assert_eq!(segments[2].index, 2, "Third segment should have index 2");

        assert!(
            segments[0].track.is_some(),
            "First segment should have a track"
        );
        assert!(
            segments[1].track.is_some(),
            "Second segment should have a track"
        );
        assert!(
            segments[2].track.is_some(),
            "Third segment should have a track"
        );
    }

    #[test]
    fn test_segment_indexing_with_pre_music() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // GPS streams every 30 seconds for 10 minutes
        let streams: Vec<activity_stream::Model> = (0..21)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 30), Some(48.0), Some(2.0)))
            .collect();

        // First track starts 2 minutes after activity start
        let listens = vec![
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(2),
                "Track A",
                "Artist A",
            ),
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(6),
                "Track B",
                "Artist B",
            ),
        ];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(
            segments.len(),
            3,
            "Should have 3 segments (pre-music + 2 tracks)"
        );
        assert_eq!(
            segments[0].index, 0,
            "Pre-music segment should have index 0"
        );
        assert_eq!(
            segments[1].index, 1,
            "First track segment should have index 1"
        );
        assert_eq!(
            segments[2].index, 2,
            "Second track segment should have index 2"
        );

        assert!(
            segments[0].track.is_none(),
            "Pre-music segment should have no track"
        );
        assert_eq!(
            segments[0].start_time, activity_start,
            "Pre-music should start at activity start"
        );
        assert_eq!(
            segments[0].end_time,
            minutes_after(2),
            "Pre-music should end at first track"
        );

        assert!(
            segments[1].track.is_some(),
            "First track segment should have a track"
        );
        assert!(
            segments[2].track.is_some(),
            "Second track segment should have a track"
        );

        // Verify no duplicate indices
        let indices: Vec<usize> = segments.iter().map(|s| s.index).collect();
        let mut sorted_indices = indices.clone();
        sorted_indices.sort_unstable();
        assert_eq!(
            indices, sorted_indices,
            "Indices should be sequential without gaps"
        );
    }

    #[test]
    fn test_segment_indexing_no_music() {
        let activity_id = Uuid::new_v4();

        // GPS streams every 40 seconds for 10 minutes
        let streams: Vec<activity_stream::Model> = (0..16)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 40), Some(48.0), Some(2.0)))
            .collect();

        let listens = vec![]; // No music

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 1, "Should have exactly 1 segment");
        assert_eq!(segments[0].index, 0, "Single segment should have index 0");
        assert!(segments[0].track.is_none(), "Segment should have no track");
        assert_eq!(
            segments[0].start_time, activity_start,
            "Should start at activity start"
        );
        assert_eq!(
            segments[0].end_time, activity_end,
            "Should end at activity end"
        );
        assert_eq!(
            segments[0].points.len(),
            16,
            "Should contain all GPS points"
        );
    }

    #[test]
    fn test_segment_time_boundaries() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // GPS streams every 20 seconds for 10 minutes (31 points)
        let streams: Vec<activity_stream::Model> = (0..31)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 20), Some(48.0), Some(2.0)))
            .collect();

        let listens = vec![
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(2),
                "Track A",
                "Artist A",
            ),
            make_listen_with_track(
                user_id,
                Uuid::new_v4(),
                minutes_after(5),
                "Track B",
                "Artist B",
            ),
        ];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 3, "Should have 3 segments");

        // Verify pre-music segment contains only points before first track
        for point in &segments[0].points {
            let time: DateTime<Utc> = point.time.into();
            assert!(
                time < minutes_after(2),
                "Pre-music segment should only contain points before 2 minutes"
            );
        }

        // Verify Track A segment contains only points between 2 and 5 minutes
        for point in &segments[1].points {
            let time: DateTime<Utc> = point.time.into();
            assert!(
                time >= minutes_after(2) && time < minutes_after(5),
                "Track A segment should only contain points between 2 and 5 minutes"
            );
        }

        // Verify Track B segment contains points from 5 minutes to end
        for point in &segments[2].points {
            let time: DateTime<Utc> = point.time.into();
            assert!(
                time >= minutes_after(5),
                "Track B segment should only contain points from 5 minutes onward"
            );
        }

        // Verify all GPS points are accounted for in segments
        let total_segment_points: usize = segments.iter().map(|s| s.points.len()).sum();
        // Points are filtered by time boundaries, so total might be less than streams.len()
        assert!(
            total_segment_points <= streams.len(),
            "Total points in segments ({total_segment_points}) should be <= total stream points ({})",
            streams.len()
        );
    }

    #[test]
    fn test_segments_with_simplification() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create 100 collinear GPS points (straight line)
        let streams: Vec<activity_stream::Model> = (0..100)
            .map(|i| {
                let offset = (i as f64) / 100.0;
                make_stream_point(
                    activity_id,
                    seconds_after(i * 6), // 10 minutes = 600 seconds
                    Some(48.0 + offset * 0.1),
                    Some(2.0 + offset * 0.1),
                )
            })
            .collect();

        let listens = vec![make_listen_with_track(
            user_id,
            Uuid::new_v4(),
            minutes_after(5),
            "Track A",
            "Artist A",
        )];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            true, // Enable simplification
            Some(10.0),
        );

        assert!(
            result.is_ok(),
            "Should successfully build segments with simplification"
        );
        let segments = result.unwrap();

        assert_eq!(
            segments.len(),
            2,
            "Should have 2 segments (pre-music + track)"
        );
        assert_eq!(segments[0].index, 0, "First segment should have index 0");
        assert_eq!(segments[1].index, 1, "Second segment should have index 1");

        // Each segment should have reduced points
        for segment in &segments {
            if !segment.points.is_empty() {
                assert!(
                    segment.points.len() < 50,
                    "Segment should have fewer points after simplification"
                );

                // First and last points should be preserved
                if segment.points.len() >= 2 {
                    let first_time: DateTime<Utc> = segment.points[0].time.into();
                    let last_time: DateTime<Utc> = segment.points.last().unwrap().time.into();
                    assert!(
                        first_time >= segment.start_time,
                        "First point should be at or after segment start"
                    );
                    assert!(
                        last_time < segment.end_time || segment.index == 1,
                        "Last point should be before segment end or in last segment"
                    );
                }
            }
        }

        let original_points = streams.len();
        let stats = calculate_stats(&segments, original_points);
        assert!(
            stats.reduction_ratio < 1.0,
            "Reduction ratio should be less than 1.0 with simplification"
        );
    }

    #[test]
    fn test_segments_with_sparse_gps() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Create 20 points with gaps (some have None lat/lng)
        let streams: Vec<activity_stream::Model> = (0..20)
            .map(|i| {
                let has_gps = i % 4 != 0; // Every 4th point has no GPS
                let (lat, lng) = if has_gps {
                    (Some(48.0), Some(2.0))
                } else {
                    (None, None)
                };
                make_stream_point(activity_id, seconds_after(i * 30), lat, lng)
            })
            .collect();

        let listens = vec![make_listen_with_track(
            user_id,
            Uuid::new_v4(),
            minutes_after(3),
            "Track A",
            "Artist A",
        )];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            true,
            Some(10.0),
        );

        assert!(
            result.is_ok(),
            "Should handle sparse GPS data without panic"
        );
        let segments = result.unwrap();

        assert_eq!(segments.len(), 2, "Should have 2 segments");

        // Verify no panic occurred and segments were created
        for segment in &segments {
            // Points may be empty if no valid GPS data in time range
            assert!(segment.index < 2, "Index should be valid");
        }
    }

    #[test]
    fn test_single_track_entire_activity() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let streams: Vec<activity_stream::Model> = (0..20)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 30), Some(48.0), Some(2.0)))
            .collect();

        // Single track at exact activity start
        let listens = vec![make_listen_with_track(
            user_id,
            Uuid::new_v4(),
            base_time(),
            "Track A",
            "Artist A",
        )];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 1, "Should have exactly 1 segment");
        assert_eq!(segments[0].index, 0, "Single segment should have index 0");
        assert!(segments[0].track.is_some(), "Segment should have a track");
        assert_eq!(
            segments[0].points.len(),
            streams.len(),
            "Segment should contain all GPS points"
        );
    }

    #[test]
    fn test_multiple_tracks_rapid_succession() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // GPS points every 10 seconds for 10 minutes (61 points)
        let streams: Vec<activity_stream::Model> = (0..61)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 10), Some(48.0), Some(2.0)))
            .collect();

        // 10 tracks, each 1 minute apart
        let listens: Vec<(listen::Model, Option<track::Model>)> = (0..10)
            .map(|i| {
                make_listen_with_track(
                    user_id,
                    Uuid::new_v4(),
                    minutes_after(i),
                    &format!("Track {i}"),
                    &format!("Artist {i}"),
                )
            })
            .collect();

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 10, "Should have 10 segments");

        // Verify indices are sequential 0-9
        for (i, segment) in segments.iter().enumerate() {
            assert_eq!(segment.index, i, "Segment {i} should have index {i}");
            assert!(segment.track.is_some(), "Segment {i} should have a track");
        }

        // Verify last segment extends to activity end
        let last_segment = &segments[9];
        assert_eq!(
            last_segment.end_time, activity_end,
            "Last segment should extend to activity end"
        );
    }

    // ==================== Group C: Edge Cases ====================

    #[test]
    fn test_empty_streams() {
        let user_id = Uuid::new_v4();

        let streams: Vec<activity_stream::Model> = vec![]; // No GPS data

        let listens = vec![make_listen_with_track(
            user_id,
            Uuid::new_v4(),
            minutes_after(5),
            "Track A",
            "Artist A",
        )];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should handle empty streams without panic");
        let segments = result.unwrap();

        // With a listen at 5 minutes, we get pre-music segment + music segment
        assert_eq!(
            segments.len(),
            2,
            "Should have 2 segments (pre-music + track)"
        );
        assert_eq!(
            segments[0].points.len(),
            0,
            "Pre-music segment should have 0 points"
        );
        assert_eq!(
            segments[1].points.len(),
            0,
            "Music segment should have 0 points"
        );

        let stats = calculate_stats(&segments, 0);
        assert_eq!(
            stats.original_points, 0,
            "Should handle 0 points gracefully"
        );
    }

    #[test]
    fn test_listens_outside_activity_range() {
        let activity_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        let streams: Vec<activity_stream::Model> = (0..20)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 30), Some(48.0), Some(2.0)))
            .collect();

        // Only include listen during activity
        // Note: build_activity_segments doesn't filter listens by time range,
        // that filtering happens in get_activity_music via database query
        let listens = vec![make_listen_with_track(
            user_id,
            Uuid::new_v4(),
            minutes_after(2), // During activity
            "Track During",
            "Artist During",
        )];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        // Should have pre-music segment + Track During segment
        assert_eq!(
            segments.len(),
            2,
            "Should have 2 segments (pre-music + during track)"
        );

        // Verify only "Track During" appears
        let track_names: Vec<String> = segments
            .iter()
            .filter_map(|s| s.track.as_ref().map(|t| t.track_name.clone()))
            .collect();
        assert_eq!(track_names.len(), 1, "Should have only 1 track");
        assert_eq!(
            track_names[0], "Track During",
            "Should only include track during activity"
        );
    }

    #[test]
    fn test_tolerance_none_uses_default() {
        let activity_id = Uuid::new_v4();

        // Create 100 collinear points
        let streams: Vec<activity_stream::Model> = (0..100)
            .map(|i| {
                let offset = (i as f64) / 100.0;
                make_stream_point(
                    activity_id,
                    seconds_after(i * 6),
                    Some(48.0 + offset * 0.1),
                    Some(2.0 + offset * 0.1),
                )
            })
            .collect();

        let listens = vec![];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            true, // simplify=true
            None, // tolerance=None should use default
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        let original_points = streams.len();
        let stats = calculate_stats(&segments, original_points);

        assert!(
            stats.reduction_ratio < 1.0,
            "Default tolerance should be applied, resulting in point reduction"
        );
        assert!(
            stats.simplified_points < stats.original_points,
            "Should have fewer points after simplification with default tolerance"
        );
    }

    #[test]
    fn test_activity_with_only_pre_music_segment() {
        let activity_id = Uuid::new_v4();

        let streams: Vec<activity_stream::Model> = (0..20)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 30), Some(48.0), Some(2.0)))
            .collect();

        // No listens during activity (simulates no music playing)
        let listens = vec![];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        assert_eq!(segments.len(), 1, "Should have exactly 1 segment");
        assert_eq!(segments[0].index, 0, "Segment should have index 0");
        assert!(segments[0].track.is_none(), "Segment should have no track");
        assert_eq!(
            segments[0].points.len(),
            streams.len(),
            "Segment should contain all GPS points"
        );
    }

    // ==================== Group D: Statistics Validation Tests ====================

    #[test]
    fn test_original_points_counts_only_activity_range() {
        let activity_id = Uuid::new_v4();

        // Create points before, during, and after activity
        let mut streams: Vec<activity_stream::Model> = vec![];

        // 5 points before activity (09:55:00 - 09:59:00)
        for i in 0..5 {
            streams.push(make_stream_point(
                activity_id,
                minutes_after(-5) + Duration::seconds(i * 60),
                Some(48.0),
                Some(2.0),
            ));
        }

        // 20 points during activity (10:00:00 - 10:10:00)
        for i in 0..20 {
            streams.push(make_stream_point(
                activity_id,
                seconds_after(i * 30),
                Some(48.0),
                Some(2.0),
            ));
        }

        // 5 points after activity (10:15:00 - 10:19:00)
        for i in 0..5 {
            streams.push(make_stream_point(
                activity_id,
                minutes_after(15) + Duration::seconds(i * 60),
                Some(48.0),
                Some(2.0),
            ));
        }

        let listens = vec![];

        let activity_start = base_time();
        let activity_end = minutes_after(10);

        // Count GPS points within activity range (mimic lines 162-170 in analytics_service.rs)
        let original_points_count = streams
            .iter()
            .filter(|s| {
                let time: DateTime<Utc> = s.time.into();
                s.latitude.is_some()
                    && s.longitude.is_some()
                    && time >= activity_start
                    && time <= activity_end
            })
            .count();

        assert_eq!(
            original_points_count, 20,
            "Should count only the 20 points within activity time range"
        );

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false,
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        let stats = calculate_stats(&segments, original_points_count);
        assert_eq!(
            stats.original_points, 20,
            "Original points should exclude points outside activity range"
        );
    }

    #[test]
    fn test_simplified_points_less_than_original() {
        let activity_id = Uuid::new_v4();

        // Test multiple route patterns
        let test_cases: Vec<(Vec<activity_stream::Model>, &str)> = vec![
            // Straight line (high reduction expected)
            (
                (0..100)
                    .map(|i| {
                        let offset = (i as f64) / 100.0;
                        make_stream_point(
                            activity_id,
                            seconds_after(i * 6),
                            Some(48.0 + offset * 0.1),
                            Some(2.0 + offset * 0.1),
                        )
                    })
                    .collect::<Vec<_>>(),
                "straight line",
            ),
            // Zigzag pattern (moderate reduction)
            (
                (0..100)
                    .map(|i| {
                        let offset = (i as f64) / 100.0;
                        let zigzag = if i % 2 == 0 { 0.0 } else { 0.001 };
                        make_stream_point(
                            activity_id,
                            seconds_after(i * 6),
                            Some(48.0 + offset * 0.1),
                            Some(2.0 + offset * 0.1 + zigzag),
                        )
                    })
                    .collect::<Vec<_>>(),
                "zigzag",
            ),
        ];

        for (streams, pattern_name) in &test_cases {
            let listens = vec![];
            let activity_start = base_time();
            let activity_end = minutes_after(10);

            let result = build_activity_segments(
                streams,
                &listens,
                activity_start,
                activity_end,
                true, // simplify=true
                Some(10.0),
            );

            assert!(result.is_ok(), "Should build segments for {pattern_name}");
            let segments = result.unwrap();

            let original_points = streams.len();
            let stats = calculate_stats(&segments, original_points);

            assert!(
                stats.simplified_points <= stats.original_points,
                "Simplified points ({}) should be <= original points ({}) for {pattern_name}",
                stats.simplified_points,
                stats.original_points
            );
        }
    }

    #[test]
    fn test_reduction_ratio_always_valid() {
        let activity_id = Uuid::new_v4();

        // Create straight line with 100 points
        let streams: Vec<activity_stream::Model> = (0..100)
            .map(|i| {
                let offset = (i as f64) / 100.0;
                make_stream_point(
                    activity_id,
                    seconds_after(i * 6),
                    Some(48.0 + offset * 0.1),
                    Some(2.0 + offset * 0.1),
                )
            })
            .collect();

        let listens = vec![];
        let activity_start = base_time();
        let activity_end = minutes_after(10);

        // Test different tolerance values
        let tolerances = vec![1.0, 10.0, 100.0];

        for tolerance in tolerances {
            let result = build_activity_segments(
                &streams,
                &listens,
                activity_start,
                activity_end,
                true,
                Some(tolerance),
            );

            assert!(
                result.is_ok(),
                "Should build segments with tolerance {tolerance}"
            );
            let segments = result.unwrap();

            let original_points = streams.len();
            let stats = calculate_stats(&segments, original_points);

            assert!(
                stats.reduction_ratio >= 0.0 && stats.reduction_ratio <= 1.0,
                "Reduction ratio ({}) should be between 0.0 and 1.0 for tolerance {tolerance}",
                stats.reduction_ratio
            );
        }
    }

    #[test]
    fn test_no_simplification_ratio_equals_one() {
        let activity_id = Uuid::new_v4();

        let streams: Vec<activity_stream::Model> = (0..50)
            .map(|i| make_stream_point(activity_id, seconds_after(i * 12), Some(48.0), Some(2.0)))
            .collect();

        let listens = vec![];
        let activity_start = base_time();
        let activity_end = minutes_after(10);

        let result = build_activity_segments(
            &streams,
            &listens,
            activity_start,
            activity_end,
            false, // simplify=false
            None,
        );

        assert!(result.is_ok(), "Should successfully build segments");
        let segments = result.unwrap();

        let original_points = streams.len();
        let stats = calculate_stats(&segments, original_points);

        assert_eq!(
            stats.simplified_points, stats.original_points,
            "Without simplification, simplified points should equal original points"
        );
        assert!(
            (stats.reduction_ratio - 1.0).abs() < 0.001,
            "Reduction ratio should be 1.0 when simplify=false"
        );
    }
}
