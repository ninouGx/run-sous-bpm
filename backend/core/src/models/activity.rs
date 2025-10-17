use chrono::{DateTime, FixedOffset};
use run_sous_bpm_integrations::strava::StravaActivityResponse;
use uuid::Uuid;

use crate::database::activity;

/// DTO for creating an activity from Strava API response
#[derive(Debug, Clone)]
pub struct CreateActivityDto {
    pub user_id: Uuid,
    pub external_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub activity_type: String,
    pub start_time: DateTime<FixedOffset>,
    pub moving_time: i32,
    pub elapsed_time: i32,
    pub timezone: String,
    pub distance: f32,
    pub total_elevation_gain: f32,
}

impl CreateActivityDto {
    /// Creates a DTO from Strava API response
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `start_date` cannot be parsed as ISO 8601 datetime
    /// - `external_id` cannot be converted to i64
    pub fn from_strava_response(
        response: StravaActivityResponse,
        user_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let start_time = DateTime::parse_from_rfc3339(&response.start_date)
            .map_err(|e| format!("Failed to parse start_date: {e}"))?;

        let external_id = i64::try_from(response.id)
            .map_err(|e| format!("Failed to convert activity ID to i64: {e}"))?;

        Ok(Self {
            user_id,
            external_id,
            name: response.name,
            description: response.description,
            activity_type: response.sport_type,
            start_time,
            moving_time: response.moving_time,
            elapsed_time: response.elapsed_time,
            timezone: response.timezone,
            distance: response.distance,
            total_elevation_gain: response.total_elevation_gain,
        })
    }

    /// Converts the DTO into a `SeaORM` `ActiveModel` for insertion
    #[must_use]
    pub fn into_active_model(self) -> activity::ActiveModel {
        use sea_orm::ActiveValue::Set;

        activity::ActiveModel {
            id: Set(Uuid::new_v4()),
            user_id: Set(self.user_id),
            external_id: Set(self.external_id),
            name: Set(self.name),
            description: Set(self.description),
            r#type: Set(self.activity_type),
            start_time: Set(self.start_time),
            moving_time: Set(self.moving_time),
            elapsed_time: Set(self.elapsed_time),
            timezone: Set(self.timezone),
            distance: Set(self.distance),
            total_elevation_gain: Set(self.total_elevation_gain),
            created_at: Set(chrono::Utc::now().into()),
            updated_at: Set(chrono::Utc::now().into()),
        }
    }
}
