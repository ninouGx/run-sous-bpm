use chrono::{DateTime, FixedOffset};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, InsertResult,
    QueryFilter, QueryOrder,
};
use uuid::Uuid;

use crate::database::{entities::prelude::Listen, listen};
use crate::models::CreateListenDto;

/// Creates a new listen record from a DTO
///
/// # Errors
///
/// Returns an error if database insert fails
pub async fn create_listen(
    db: &DatabaseConnection,
    dto: CreateListenDto,
) -> Result<listen::Model, DbErr> {
    let active_model = dto.into_active_model();
    active_model.insert(db).await
}

/// Batch inserts multiple listen records
/// Uses `SeaORM`'s bulk insert for efficiency
///
/// # Errors
///
/// Returns an error if database insert fails
pub async fn batch_create_listens(
    db: &DatabaseConnection,
    listens: Vec<listen::ActiveModel>,
) -> Result<InsertResult<listen::ActiveModel>, DbErr> {
    Listen::insert_many(listens).exec(db).await
}

/// Retrieves listens for a user within a specific time range
/// Ordered by `played_at` ascending (chronological order)
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_listens_by_user_time_range(
    db: &DatabaseConnection,
    user_id: Uuid,
    start_time: DateTime<FixedOffset>,
    end_time: DateTime<FixedOffset>,
) -> Result<Vec<listen::Model>, DbErr> {
    Listen::find()
        .filter(listen::Column::UserId.eq(user_id))
        .filter(listen::Column::PlayedAt.gte(start_time))
        .filter(listen::Column::PlayedAt.lte(end_time))
        .order_by_asc(listen::Column::PlayedAt)
        .all(db)
        .await
}

/// Retrieves all listens for a specific user
/// Ordered by `played_at` descending (most recent first)
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_listens_by_user(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<listen::Model>, DbErr> {
    Listen::find()
        .filter(listen::Column::UserId.eq(user_id))
        .order_by_desc(listen::Column::PlayedAt)
        .all(db)
        .await
}

/// Retrieves a specific listen by its internal UUID
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_listen_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<listen::Model>, DbErr> {
    Listen::find()
        .filter(listen::Column::Id.eq(id))
        .one(db)
        .await
}

/// Retrieves all listens for a specific track
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_listens_by_track(
    db: &DatabaseConnection,
    track_id: Uuid,
) -> Result<Vec<listen::Model>, DbErr> {
    Listen::find()
        .filter(listen::Column::TrackId.eq(track_id))
        .order_by_desc(listen::Column::PlayedAt)
        .all(db)
        .await
}

/// Deletes a listen by its internal UUID
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - Listen not found
pub async fn delete_listen(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
    let listen = get_listen_by_id(db, id).await?;

    match listen {
        Some(l) => {
            let active_model: listen::ActiveModel = l.into();
            active_model.delete(db).await?;
            Ok(())
        }
        None => Err(DbErr::RecordNotFound("Listen not found".into())),
    }
}

/// Deletes all listens for a user within a time range
/// Useful for re-syncing a specific time period
///
/// # Errors
///
/// Returns an error if database operation fails
pub async fn delete_listens_by_user_time_range(
    db: &DatabaseConnection,
    user_id: Uuid,
    start_time: DateTime<FixedOffset>,
    end_time: DateTime<FixedOffset>,
) -> Result<u64, DbErr> {
    let result = Listen::delete_many()
        .filter(listen::Column::UserId.eq(user_id))
        .filter(listen::Column::PlayedAt.gte(start_time))
        .filter(listen::Column::PlayedAt.lte(end_time))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}
