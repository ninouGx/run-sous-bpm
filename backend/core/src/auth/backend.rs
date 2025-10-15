use crate::{
    auth::verify_password,
    database::{entities::user, user::Entity},
};
use axum_login::{AuthnBackend, UserId};
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, QueryFilter};
use serde::Deserialize;
use validator::Validate;

#[derive(Clone, Deserialize, Validate)]
pub struct Credentials {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Clone)]
pub struct AuthBackend {
    db: DatabaseConnection,
}

impl AuthBackend {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

impl AuthnBackend for AuthBackend {
    type User = user::Model;

    type Credentials = Credentials;

    type Error = DbErr;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let user = Entity::find()
            .filter(user::Column::Email.eq(creds.email))
            .one(&self.db)
            .await?;
        if let Some(user) = user {
            match verify_password(
                &creds.password,
                user.password_hash.as_deref().unwrap_or_default(),
            ) {
                Ok(true) => Ok(Some(user)),
                Ok(false) => Ok(None),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = Entity::find_by_id(*user_id).one(&self.db).await?;
        Ok(user)
    }
}
