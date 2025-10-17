use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::database::{activity, entities::prelude::Activity};
use crate::models::CreateActivityDto;

/// Creates a new activity from a DTO
///
/// # Errors
///
/// Returns an error if database insert fails
pub async fn create_activity(
    db: &DatabaseConnection,
    dto: CreateActivityDto,
) -> Result<activity::Model, DbErr> {
    let active_model = dto.into_active_model();
    active_model.insert(db).await
}

/// Creates or updates an activity based on `external_id` (Strava ID)
/// If an activity with the same `external_id` exists for this user, it updates it
///
/// # Errors
///
/// Returns an error if database operation fails
pub async fn upsert_activity(
    db: &DatabaseConnection,
    dto: CreateActivityDto,
) -> Result<activity::Model, DbErr> {
    // Check if activity already exists
    let existing = get_activity_by_external_id(db, dto.user_id, dto.external_id).await?;

    match existing {
        Some(existing_activity) => {
            // Update existing activity
            let mut active_model: activity::ActiveModel = existing_activity.into();
            active_model.name = Set(dto.name);
            active_model.description = Set(dto.description);
            active_model.r#type = Set(dto.activity_type);
            active_model.start_time = Set(dto.start_time);
            active_model.moving_time = Set(dto.moving_time);
            active_model.elapsed_time = Set(dto.elapsed_time);
            active_model.timezone = Set(dto.timezone);
            active_model.distance = Set(dto.distance);
            active_model.total_elevation_gain = Set(dto.total_elevation_gain);
            active_model.updated_at = Set(chrono::Utc::now().into());

            active_model.update(db).await
        }
        None => {
            // Create new activity
            create_activity(db, dto).await
        }
    }
}

/// Retrieves an activity by `external_id` (Strava ID) for a specific user
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_activity_by_external_id(
    db: &DatabaseConnection,
    user_id: Uuid,
    external_id: i64,
) -> Result<Option<activity::Model>, DbErr> {
    Activity::find()
        .filter(activity::Column::UserId.eq(user_id))
        .filter(activity::Column::ExternalId.eq(external_id))
        .one(db)
        .await
}

/// Retrieves an activity by its internal UUID
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_activity_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<activity::Model>, DbErr> {
    Activity::find()
        .filter(activity::Column::Id.eq(id))
        .one(db)
        .await
}

/// Retrieves all activities for a specific user, ordered by start time (descending)
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_activities_by_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<activity::Model>, DbErr> {
    Activity::find()
        .filter(activity::Column::UserId.eq(user_id))
        .order_by_desc(activity::Column::StartTime)
        .all(db)
        .await
}

/// Deletes an activity by its internal UUID
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - Activity not found
pub async fn delete_activity(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
    let activity = get_activity_by_id(db, id).await?;

    match activity {
        Some(a) => {
            let active_model: activity::ActiveModel = a.into();
            active_model.delete(db).await?;
            Ok(())
        }
        None => Err(DbErr::RecordNotFound("Activity not found".into())),
    }
}
