use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::database::user;

/// Creates a new user with email and password hash
///
/// # Errors
///
/// Returns an error if database insert fails
pub async fn create_user(
    db: &DatabaseConnection,
    email: String,
    password_hash: String,
) -> Result<user::Model, DbErr> {
    let new_user = user::ActiveModel {
        email: Set(email),
        password_hash: Set(Some(password_hash)),
        ..Default::default()
    };

    new_user.insert(db).await
}

/// Retrieves a user by email address
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

/// Retrieves a user by ID
///
/// # Errors
///
/// Returns an error if database query fails
pub async fn get_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Id.eq(id))
        .one(db)
        .await
}

/// Updates a user's email address with uniqueness validation
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - User not found
/// - New email is already in use by another user
pub async fn update_user_email(
    db: &DatabaseConnection,
    id: Uuid,
    new_email: String,
) -> Result<user::Model, DbErr> {
    let user = get_user_by_id(db, id).await?;
    let existing_user = get_user_by_email(db, new_email.clone()).await?;

    if existing_user.is_some() {
        return Err(DbErr::Custom("Email already in use".into()));
    }

    match user {
        Some(mut u) => {
            u.email = new_email;
            let active_model: user::ActiveModel = u.into();
            active_model.update(db).await
        }
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}

/// Updates a user's lastfm username
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - User not found
pub async fn update_user_lastfm_username(
    db: &DatabaseConnection,
    id: Uuid,
    new_lastfm_username: String,
) -> Result<user::Model, DbErr> {
    let user = get_user_by_id(db, id).await?;

    match user {
        Some(u) => {
            let mut active_model: user::ActiveModel = u.into();
            active_model.lastfm_username = Set(Some(new_lastfm_username));
            active_model.update(db).await
        }
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}

/// Deletes a user by ID
///
/// # Errors
///
/// Returns an error if:
/// - Database query fails
/// - User not found
pub async fn delete_user(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
    let user = get_user_by_id(db, id).await?;

    match user {
        Some(u) => {
            let active_model: user::ActiveModel = u.into();
            active_model.delete(db).await?;
            Ok(())
        }
        None => Err(DbErr::RecordNotFound("User not found".into())),
    }
}
