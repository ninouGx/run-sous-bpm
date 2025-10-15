use dotenvy::dotenv;
use oauth2::{AuthType, AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl};
use serde::{Deserialize, Serialize};
use std::env;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OAuthProvider {
    Strava,
    Spotify,
}

pub struct ClientInfo {
    pub(crate) client_id: ClientId,
    pub(crate) client_secret: ClientSecret,
    pub(crate) auth_url: AuthUrl,
    pub(crate) token_url: TokenUrl,
    pub(crate) redirect_url: RedirectUrl,
    pub(crate) scopes: Vec<Scope>,
    pub(crate) auth_type: AuthType,
}

impl ClientInfo {
    fn retrieve_env_var(var_name: &str) -> String {
        dotenv().ok();
        env::var(var_name).unwrap_or_else(|_| panic!("{var_name} must be set in .env file"))
    }

    /// Creates OAuth client configuration from provider type
    ///
    /// # Panics
    ///
    /// Panics if:
    /// - Required environment variables are not set (`CLIENT_ID`, `CLIENT_SECRET`, `REDIRECT_URI`, `AUTH_URL`, `TOKEN_URL`)
    /// - URL parsing fails for `auth_url`, `token_url`, or `redirect_url`
    #[must_use]
    pub fn from_provider(provider: OAuthProvider) -> Self {
        let prefix = provider.to_string().to_uppercase();
        let client_id = ClientId::new(Self::retrieve_env_var(&format!("{prefix}_CLIENT_ID")));
        let client_secret =
            ClientSecret::new(Self::retrieve_env_var(&format!("{prefix}_CLIENT_SECRET")));
        let redirect_url = RedirectUrl::new(Self::retrieve_env_var("REDIRECT_URI"));
        let auth_url = AuthUrl::new(Self::retrieve_env_var(&format!("{prefix}_AUTH_URL")));
        let token_url = TokenUrl::new(Self::retrieve_env_var(&format!("{prefix}_TOKEN_URL")));
        let scopes = match provider {
            OAuthProvider::Strava => vec![Scope::new("activity:read_all".to_string())],
            OAuthProvider::Spotify => vec![Scope::new("user-read-recently-played".to_string())],
        };
        let auth_type = AuthType::RequestBody;

        ClientInfo {
            client_id,
            client_secret,
            auth_url: auth_url.expect("AuthUrl must be valid"),
            token_url: token_url.expect("TokenUrl must be valid"),
            redirect_url: redirect_url.expect("RedirectUrl must be valid"),
            scopes,
            auth_type,
        }
    }

    // Public getters for accessing fields from outside the crate
    #[must_use]
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    #[must_use]
    pub fn client_secret(&self) -> &ClientSecret {
        &self.client_secret
    }

    #[must_use]
    pub fn auth_url(&self) -> &AuthUrl {
        &self.auth_url
    }

    #[must_use]
    pub fn token_url(&self) -> &TokenUrl {
        &self.token_url
    }

    #[must_use]
    pub fn redirect_url(&self) -> &RedirectUrl {
        &self.redirect_url
    }

    #[must_use]
    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    #[must_use]
    pub fn auth_type(&self) -> &AuthType {
        &self.auth_type
    }
}
