pub mod backend;
pub mod password;

pub use backend::*;
pub use password::*;

use axum_login::AuthUser;
use uuid::Uuid;

use crate::database::user;

impl AuthUser for user::Model {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.password_hash
            .as_ref()
            .map_or(&[], std::string::String::as_bytes)
    }
}
