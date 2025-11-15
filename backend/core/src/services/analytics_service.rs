use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::info;
use uuid::Uuid;

use crate::{
    database::{
        entities::prelude::{Listen, Track},
        get_activity_by_id, get_listens_by_user_time_range, get_user_by_id,
        listen::{self},
        track,
    },
    services::sync_lastfm_for_time_range,
};

/// Retrieves music tracks played during a specific activity
///
/// # Arguments
/// * `db` - Database connection
/// * `user_id` - ID of the user
/// * `activity_id` - ID of the activity
///
/// # Errors
///
/// Returns an error if:
/// - Activity is not found in the database
/// - User is not found in the database
/// - User does not have a Last.fm username configured
/// - Last.fm API request fails
/// - Database query fails
pub async fn get_activity_music(
    db: &DatabaseConnection,
    user_id: Uuid,
    activity_id: Uuid,
) -> Result<Vec<(listen::Model, track::Model)>, Box<dyn std::error::Error>> {
    let activity = get_activity_by_id(db, activity_id)
        .await?
        .ok_or("Activity not found")?;

    if activity.user_id != user_id {
        return Err("Activity does not belong to the user".into());
    }

    let end_time =
        activity.start_time + chrono::Duration::seconds(i64::from(activity.elapsed_time));
    let wide_start_time = activity.start_time;

    info!(
        user_id = %user_id,
        activity_id = %activity_id,
        start_time = %activity.start_time,
        wide_start_time = %wide_start_time,
        end_time = %end_time,
        "Fetching listens for activity time range"
    );

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
    let listens_with_tracks = Listen::find()
        .filter(listen::Column::UserId.eq(user_id))
        .filter(listen::Column::PlayedAt.gte(wide_start_time))
        .filter(listen::Column::PlayedAt.lte(end_time))
        .find_also_related(Track)
        .all(db)
        .await?;

    Ok(listens_with_tracks
        .into_iter()
        .filter_map(|(listen, track)| track.map(|t| (listen, t)))
        .collect())
}
