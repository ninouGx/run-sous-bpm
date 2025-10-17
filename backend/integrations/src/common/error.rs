#[derive(Debug)]
pub enum IntegrationError {
    Http(reqwest::Error),
    OAuth(String),
    Database(sea_orm::DbErr),
    TokenNotFound,
    TokenExpired,
    RefreshFailed(String),
    Deserialization(String),
    Other(String),
}

impl std::fmt::Display for IntegrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "HTTP error: {e}"),
            Self::OAuth(msg) => write!(f, "OAuth error: {msg}"),
            Self::Database(e) => write!(f, "Database error: {e}"),
            Self::TokenNotFound => write!(f, "OAuth token not found for user"),
            Self::TokenExpired => write!(f, "OAuth token expired and no refresh token available"),
            Self::RefreshFailed(msg) => write!(f, "Token refresh failed: {msg}"),
            Self::Deserialization(msg) => write!(f, "Failed to deserialize response: {msg}"),
            Self::Other(msg) => write!(f, "Integration error: {msg}"),
        }
    }
}

impl std::error::Error for IntegrationError {}

impl From<reqwest::Error> for IntegrationError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

impl From<sea_orm::DbErr> for IntegrationError {
    fn from(err: sea_orm::DbErr) -> Self {
        Self::Database(err)
    }
}

impl From<Box<dyn std::error::Error>> for IntegrationError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for IntegrationError {
    fn from(err: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::Other(err.to_string())
    }
}
