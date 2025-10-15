use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter, prelude::DateTimeWithTimeZone,
};
use uuid::Uuid;

use crate::config::OAuthProvider;
use crate::database::oauth_token;

pub async fn create_oauth_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: OAuthProvider,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<DateTimeWithTimeZone>,
    scopes: Option<Vec<String>>,
) -> Result<oauth_token::Model, DbErr> {
    let new_token = oauth_token::ActiveModel {
        user_id: Set(user_id),
        provider: Set(provider.to_string()),
        access_token: Set(access_token),
        refresh_token: Set(refresh_token),
        expires_at: Set(expires_at),
        scopes: Set(scopes),
        ..Default::default()
    };

    new_token.insert(db).await
}

pub async fn get_oauth_token_by_provider(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: OAuthProvider,
) -> Result<Option<oauth_token::Model>, DbErr> {
    oauth_token::Entity::find()
        .filter(oauth_token::Column::UserId.eq(user_id))
        .filter(oauth_token::Column::Provider.eq(provider.to_string()))
        .one(db)
        .await
}

pub async fn upsert_oauth_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: OAuthProvider,
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<DateTimeWithTimeZone>,
    scopes: Option<Vec<String>>,
) -> Result<oauth_token::Model, DbErr> {
    let token = get_oauth_token_by_provider(db, user_id, provider).await?;
    match token {
        Some(existing_token) => {
            let mut active_token: oauth_token::ActiveModel = existing_token.into();
            active_token.access_token = Set(access_token);
            active_token.refresh_token = Set(refresh_token);
            active_token.expires_at = Set(expires_at);
            active_token.scopes = Set(scopes);
            active_token.updated_at = Set(chrono::Utc::now().into());

            active_token.update(db).await
        }
        None => {
            create_oauth_token(
                db,
                user_id,
                provider,
                access_token,
                refresh_token,
                expires_at,
                scopes,
            )
            .await
        }
    }
}

pub async fn delete_oauth_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    provider: OAuthProvider,
) -> Result<(), DbErr> {
    let token = get_oauth_token_by_provider(db, user_id, provider).await?;

    match token {
        Some(t) => {
            let active_model: oauth_token::ActiveModel = t.into();
            active_model.delete(db).await?;
            Ok(())
        }
        None => Err(DbErr::RecordNotFound("OAuth token not found".into())),
    }
}
