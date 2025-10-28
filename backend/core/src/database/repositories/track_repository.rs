use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::database::{entities::prelude::Track, track};
use crate::models::CreateTrackDto;

/// Creates a new track from a DTO
///
/// # Errors
///
/// Returns an error if database insert fails
pub async fn create_track(
    db: &DatabaseConnection,
    dto: CreateTrackDto,
) -> Result<track::Model, DbErr> {
    let active_model = dto.into_active_model();
    active_model.insert(db).await
}

/// Creates or updates a track based on `(artist_name, track_name)` unique constraint
/// If a track with the same artist and name exists, it returns the existing track
///
/// # Errors
///
/// Returns an error if database operation fails
pub async fn upsert_track(
    db: &DatabaseConnection,
    dto: CreateTrackDto,
) -> Result<track::Model, DbErr> {
    // Check if track already exists
    let existing = get_track_by_metadata(db, &dto.artist_name, &dto.track_name).await?;

    match existing {
        Some(existing_track) => {
            // Track already exists, optionally update metadata if needed
            // For now, just return the existing track
            // Future: Could update MBIDs or URL if they were empty before
            Ok(existing_track)
        }
        None => {
            // Create new track
            create_track(db, dto).await
        }
    }
}

/// Retrieves a track by artist name and track name
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_track_by_metadata(
    db: &DatabaseConnection,
    artist_name: &str,
    track_name: &str,
) -> Result<Option<track::Model>, DbErr> {
    Track::find()
        .filter(track::Column::ArtistName.eq(artist_name))
        .filter(track::Column::TrackName.eq(track_name))
        .one(db)
        .await
}

/// Retrieves a track by its internal UUID
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_track_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<track::Model>, DbErr> {
    Track::find().filter(track::Column::Id.eq(id)).one(db).await
}

/// Retrieves a track by `MusicBrainz` track ID
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_track_by_mbid(
    db: &DatabaseConnection,
    track_mbid: &str,
) -> Result<Option<track::Model>, DbErr> {
    Track::find()
        .filter(track::Column::TrackMbid.eq(track_mbid))
        .one(db)
        .await
}

/// Deletes a track by its internal UUID
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - Track not found
pub async fn delete_track(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
    let track = get_track_by_id(db, id).await?;

    match track {
        Some(t) => {
            let active_model: track::ActiveModel = t.into();
            active_model.delete(db).await?;
            Ok(())
        }
        None => Err(DbErr::RecordNotFound("Track not found".into())),
    }
}
