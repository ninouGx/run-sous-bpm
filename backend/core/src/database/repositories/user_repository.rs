use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::database::user;

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

pub async fn get_user_by_email(
    db: &DatabaseConnection,
    email: String,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await
}

pub async fn get_user_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<user::Model>, DbErr> {
    user::Entity::find()
        .filter(user::Column::Id.eq(id))
        .one(db)
        .await
}

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
