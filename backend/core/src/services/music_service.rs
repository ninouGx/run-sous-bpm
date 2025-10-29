use chrono::TimeZone;
use run_sous_bpm_integrations::lastfm::LastFmClient;
use sea_orm::DatabaseConnection;
use tracing::info;

use crate::{
    database::{ batch_create_listens, listen, upsert_track },
    models::{ CreateListenDto, CreateTrackDto },
};

/// Syncs Last.fm listening history for a specific time range (e.g., during an activity)
///
/// # Arguments
/// * `user_id` - UUID of the user
/// * `lastfm_username` - Last.fm username to fetch data for
/// * `start_timestamp` - Unix timestamp (seconds) for start of range
/// * `end_timestamp` - Unix timestamp (seconds) for end of range
/// * `lastfm_client` - Last.fm API client instance
/// * `db_connection` - Database connection
///
/// # Errors
///
/// Returns an error if:
/// - Last.fm API request fails
/// - Track/listen DTO conversion fails
/// - Database insertion fails
///
/// # Panics
/// Panics if the provided timestamps are out of valid date range
///
/// # Returns
/// Vector of saved listen records
pub async fn sync_lastfm_for_time_range(
    user_id: uuid::Uuid,
    lastfm_username: &str,
    start_timestamp: i64,
    end_timestamp: i64,
    db_connection: &DatabaseConnection
) -> Result<Vec<listen::Model>, Box<dyn std::error::Error>> {
    let lastfm_client = LastFmClient::new();

    let lastfm_tracks = lastfm_client.get_tracks_in_time_range(
        lastfm_username,
        start_timestamp,
        end_timestamp
    ).await?;

    info!(
        user_id = %user_id,
        lastfm_username = lastfm_username,
        start_timestamp = start_timestamp,
        end_timestamp = end_timestamp,
        tracks_fetched = lastfm_tracks.len(),
        "Fetched Last.fm tracks for time range"
    );

    if lastfm_tracks.is_empty() {
        info!("No tracks found in time range");
        return Ok(Vec::new());
    }

    let mut listen_models = Vec::new();

    for lastfm_track in lastfm_tracks {
        let Some(date) = &lastfm_track.date else {
            continue;
        };

        // Create track DTO and upsert (deduplicates by artist+track name)
        let track_dto = CreateTrackDto::from_lastfm_track(&lastfm_track);
        let saved_track = upsert_track(db_connection, track_dto).await?;

        let listen_dto = CreateListenDto::new(user_id, saved_track.id, date.uts);

        listen_models.push(listen_dto.into_active_model());
    }

    let insert_count = listen_models.len();
    batch_create_listens(db_connection, listen_models).await?;

    info!(
        user_id = %user_id,
        listens_saved = insert_count,
        "Successfully synced Last.fm listening history"
    );

    let saved_listens = crate::database::get_listens_by_user_time_range(
        db_connection,
        user_id,
        chrono::Utc
            .timestamp_opt(start_timestamp, 0)
            .single()
            .expect("Invalid start timestamp")
            .fixed_offset(),
        chrono::Utc
            .timestamp_opt(end_timestamp, 0)
            .single()
            .expect("Invalid end timestamp")
            .fixed_offset()
    ).await?;

    Ok(saved_listens)
}
