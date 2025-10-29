use lastfm_client::types::RecentTrack;
use uuid::Uuid;

use crate::database::track;

/// DTO for creating a track from Last.fm API response
#[derive(Debug, Clone)]
pub struct CreateTrackDto {
    pub artist_name: String,
    pub track_name: String,
    pub album_name: Option<String>,
    pub artist_mbid: Option<String>,
    pub track_mbid: Option<String>,
    pub album_mbid: Option<String>,
    pub lastfm_url: Option<String>,
}

impl CreateTrackDto {
    /// Creates a DTO from Last.fm `RecentTrack`
    ///
    /// # Arguments
    /// * `track` - The Last.fm recent track response
    ///
    /// # Returns
    /// * `Self` - The created DTO
    #[must_use]
    pub fn from_lastfm_track(track: &RecentTrack) -> Self {
        // Last.fm returns empty strings for missing MBIDs, convert to None
        let artist_mbid = if track.artist.mbid.is_empty() {
            None
        } else {
            Some(track.artist.mbid.clone())
        };

        let track_mbid = if track.mbid.is_empty() {
            None
        } else {
            Some(track.mbid.clone())
        };

        let album_mbid = if track.album.mbid.is_empty() {
            None
        } else {
            Some(track.album.mbid.clone())
        };

        // Album name can be empty string in Last.fm, treat as None
        let album_name = if track.album.text.is_empty() {
            None
        } else {
            Some(track.album.text.clone())
        };

        Self {
            artist_name: track.artist.text.clone(),
            track_name: track.name.clone(),
            album_name,
            artist_mbid,
            track_mbid,
            album_mbid,
            lastfm_url: Some(track.url.clone()),
        }
    }

    /// Converts the DTO into a `SeaORM` `ActiveModel` for insertion
    #[must_use]
    pub fn into_active_model(self) -> track::ActiveModel {
        use sea_orm::ActiveValue::Set;

        track::ActiveModel {
            id: Set(Uuid::new_v4()),
            artist_name: Set(self.artist_name),
            track_name: Set(self.track_name),
            album_name: Set(self.album_name),
            artist_mbid: Set(self.artist_mbid),
            track_mbid: Set(self.track_mbid),
            album_mbid: Set(self.album_mbid),
            lastfm_url: Set(self.lastfm_url),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        }
    }
}
