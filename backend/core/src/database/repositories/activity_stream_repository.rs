use sea_orm::{
    DatabaseConnection,
    DbErr,
    EntityTrait,
    TransactionTrait,
    ColumnTrait,
    QueryFilter,
    QueryOrder,
};
use uuid::Uuid;

use crate::database::activity_stream::{ ActiveModel, Model };
use crate::database::entities::prelude::ActivityStream;

/// Creates or updates activity streams in batch
/// # Errors
/// Returns an error if database operation fails
pub async fn batch_upsert_activity_streams(
    db: &DatabaseConnection,
    models: Vec<ActiveModel>
) -> Result<(), DbErr> {
    const CHUNK_SIZE: usize = 5000;

    let transaction = db.begin().await?;
    for chunk in models.chunks(CHUNK_SIZE) {
        ActivityStream::insert_many(chunk.to_vec()).exec(&transaction).await?;
    }
    transaction.commit().await?;

    Ok(())
}

/// Retrieves all activity streams for a specific activity, ordered by time
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_activity_streams(
    db: &DatabaseConnection,
    activity_id: Uuid
) -> Result<Vec<Model>, DbErr> {
    use crate::database::activity_stream;

    ActivityStream::find()
        .filter(activity_stream::Column::ActivityId.eq(activity_id))
        .order_by_asc(activity_stream::Column::Time)
        .all(db).await
}
