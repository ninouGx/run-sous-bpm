use run_sous_bpm_integrations::strava::{StravaActivityStreamsParams, StravaApiClient};
use sea_orm::DatabaseConnection;
use tracing::info;

use crate::{
    config::OAuthProvider,
    crypto::EncryptionService,
    database::{activity, activity_repository, batch_upsert_activity_streams, upsert_activity},
    models::{CreateActivityDto, ValidatedActivityStreams},
    services::get_valid_token,
};

/// Syncs Strava activities for a user and stores them in the database
///
/// # Errors
///
/// Returns an error if:
/// - OAuth token retrieval fails
/// - Strava API request fails
/// - Activity DTO conversion fails
/// - Database insertion fails
pub async fn sync_strava_activities(
    user_id: uuid::Uuid,
    strava_client: &StravaApiClient,
    db_connection: &DatabaseConnection,
    encryption: &EncryptionService,
) -> Result<Vec<activity::Model>, Box<dyn std::error::Error>> {
    let token = get_valid_token(db_connection, user_id, OAuthProvider::Strava, encryption).await?;

    let strava_activities = strava_client.get_athlete_activities(&token, None).await?;

    let mut saved_activities = Vec::new();

    // Convert and save each activity
    for strava_activity in strava_activities {
        // Convert Strava response to DTO
        let dto = CreateActivityDto::from_strava_response(strava_activity, user_id)?;

        // Save or update activity in database
        let saved_activity = upsert_activity(db_connection, dto).await?;
        saved_activities.push(saved_activity);
    }

    Ok(saved_activities)
}

/// Syncs activity stream data for a specific Strava activity
///
/// # Errors
///
/// Returns an error if:
/// - OAuth token retrieval fails
/// - Activity not found in database
/// - Strava API request fails
/// - Stream validation fails
/// - Database insertion fails
pub async fn sync_strava_activity_streams(
    user_id: uuid::Uuid,
    external_id: i64,
    strava_client: &StravaApiClient,
    db_connection: &DatabaseConnection,
    encryption: &EncryptionService,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = get_valid_token(db_connection, user_id, OAuthProvider::Strava, encryption).await?;
    let keys = &[
        "time",
        "distance",
        "latlng",
        "altitude",
        "heart_rate",
        "cadence",
        "watts",
        "velocity_smooth",
        "temperature",
    ];
    let params = StravaActivityStreamsParams::new(keys);
    let streams = strava_client
        .get_activity_streams(&token, external_id, params)
        .await?;

    let activity =
        activity_repository::get_activity_by_external_id(db_connection, user_id, external_id)
            .await?
            .ok_or("Activity not found")?;
    let dto = ValidatedActivityStreams::from_strava_response(streams, activity.id)?;

    let models = dto.into_active_models(activity.start_time);
    let count = models.len();

    batch_upsert_activity_streams(db_connection, models).await?;

    info!(
        user_id = %user_id,
        activity_id = %activity.id,
        external_id = external_id,
        points = count,
        "Successfully synced activity streams"
    );
    Ok(())
}

/// Syncs activity streams for all activities of a user
/// # Errors
///
/// Returns an error if:
/// - OAuth token retrieval fails
/// - Strava API request fails
/// - Stream validation fails
/// - Database insertion fails
pub async fn sync_all_strava_activity_streams(
    user_id: uuid::Uuid,
    strava_client: &StravaApiClient,
    db_connection: &DatabaseConnection,
    encryption: &EncryptionService,
) -> Result<(), Box<dyn std::error::Error>> {
    let activities = activity_repository::get_activities_by_user(db_connection, user_id).await?;

    for activity in activities {
        if let Err(e) = sync_strava_activity_streams(
            user_id,
            activity.external_id,
            strava_client,
            db_connection,
            encryption,
        )
        .await
        {
            info!(
                user_id = %user_id,
                activity_id = %activity.id,
                external_id = activity.external_id,
                error = %e,
                "Failed to sync activity streams"
            );
        }
    }

    Ok(())
}
