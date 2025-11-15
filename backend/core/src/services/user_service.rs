use sea_orm::DatabaseConnection;
use tracing::info;

use crate::database::user_repository;

/// Updates a user's Last.fm username after validating it exists
///
/// # Arguments
/// * `user_id` - UUID of the user to update
/// * `lastfm_username` - Last.fm username to set for the user
/// * `db_connection` - Database connection reference
///
/// # Errors
/// Returns an error if:
/// - The Last.fm username does not exist
/// - Database update fails
pub async fn update_valid_user_lastfm_username(
    user_id: uuid::Uuid,
    lastfm_username: String,
    db_connection: &DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let last_fm_client = run_sous_bpm_integrations::lastfm::LastFmClient::new();
    let is_valid = last_fm_client.is_username_valid(&lastfm_username).await;
    if !is_valid {
        return Err("Invalid Last.fm username".into());
    }

    user_repository::update_user_lastfm_username(db_connection, user_id, lastfm_username.clone())
        .await?;

    info!(
        user_id = %user_id,
        lastfm_username = ?lastfm_username,
        "Updated user's Last.fm username"
    );

    Ok(())
}
