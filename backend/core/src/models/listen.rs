use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use uuid::Uuid;

use crate::database::listen;

/// DTO for creating a listen (listening event) record
#[derive(Debug, Clone)]
pub struct CreateListenDto {
    pub user_id: Uuid,
    pub track_id: Uuid,
    pub played_at: DateTime<FixedOffset>,
}

impl CreateListenDto {
    /// Creates a new listen DTO
    ///
    /// # Arguments
    /// * `user_id` - UUID of the user who listened to the track
    /// * `track_id` - UUID of the track that was played
    /// * `played_at_timestamp` - Unix timestamp (seconds since epoch) when the track was played
    ///
    /// # Panics
    /// Panics if the timestamp cannot be converted to a valid date (out of range)
    ///
    /// # Returns
    /// * `Self` - The created DTO
    #[must_use]
    pub fn new(user_id: Uuid, track_id: Uuid, played_at_timestamp: u32) -> Self {
        // Convert Unix timestamp to DateTime with UTC timezone
        let played_at = Utc
            .timestamp_opt(i64::from(played_at_timestamp), 0)
            .single()
            .expect("Invalid timestamp")
            .fixed_offset();

        Self {
            user_id,
            track_id,
            played_at,
        }
    }

    /// Converts the DTO into a `SeaORM` `ActiveModel` for insertion
    #[must_use]
    pub fn into_active_model(self) -> listen::ActiveModel {
        use sea_orm::ActiveValue::Set;

        listen::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(self.user_id),
            track_id: Set(self.track_id),
            played_at: Set(self.played_at),
            created_at: Set(chrono::Utc::now().into()),
        }
    }
}
