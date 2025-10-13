use oauth2::{ AuthType, AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl };
use serde::{ Deserialize, Serialize };
use strum::{ Display, EnumString };
use dotenvy::dotenv;
use std::env;

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
        env::var(var_name).expect(&format!("{} must be set in .env file", var_name))
    }

    pub fn from_provider(provider: OAuthProvider) -> Self {
        let prefix = provider.to_string().to_uppercase();
        let client_id = ClientId::new(Self::retrieve_env_var(&format!("{}_CLIENT_ID", prefix)));
        let client_secret = ClientSecret::new(
            Self::retrieve_env_var(&format!("{}_CLIENT_SECRET", prefix))
        );
        let redirect_url = RedirectUrl::new(Self::retrieve_env_var("REDIRECT_URI"));
        let auth_url = AuthUrl::new(Self::retrieve_env_var(&format!("{}_AUTH_URL", prefix)));
        let token_url = TokenUrl::new(Self::retrieve_env_var(&format!("{}_TOKEN_URL", prefix)));
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
    pub fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    pub fn client_secret(&self) -> &ClientSecret {
        &self.client_secret
    }

    pub fn auth_url(&self) -> &AuthUrl {
        &self.auth_url
    }

    pub fn token_url(&self) -> &TokenUrl {
        &self.token_url
    }

    pub fn redirect_url(&self) -> &RedirectUrl {
        &self.redirect_url
    }

    pub fn scopes(&self) -> &[Scope] {
        &self.scopes
    }

    pub fn auth_type(&self) -> &AuthType {
        &self.auth_type
    }
}
