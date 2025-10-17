use sea_orm::{DatabaseConnection, DbErr, EntityTrait, TransactionTrait};

use crate::database::activity_stream::ActiveModel;
use crate::database::entities::prelude::ActivityStream;

/// Creates or updates activity streams in batch
/// # Errors
/// Returns an error if database operation fails
pub async fn batch_upsert_activity_streams(
    db: &DatabaseConnection,
    models: Vec<ActiveModel>,
) -> Result<(), DbErr> {
    const CHUNK_SIZE: usize = 5000;

    let transaction = db.begin().await?;
    for chunk in models.chunks(CHUNK_SIZE) {
        ActivityStream::insert_many(chunk.to_vec())
            .exec(&transaction)
            .await?;
    }
    transaction.commit().await?;

    Ok(())
}
