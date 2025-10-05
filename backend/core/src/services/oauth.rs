use oauth2::url::Url;
use oauth2::{
    AuthType,
    AuthUrl,
    AuthorizationCode,
    ClientId,
    ClientSecret,
    CsrfToken,
    PkceCodeChallenge,
    RedirectUrl,
    Scope,
    TokenResponse,
    TokenUrl,
};
use oauth2::basic::BasicClient;
use oauth2::reqwest;
use dotenv::dotenv;

use std::env;

use crate::models::OAuthProvider;

// Type alias for a fully configured OAuth client with auth and token endpoints set
type ConfiguredClient = oauth2::Client<
    oauth2::basic::BasicErrorResponse,
    oauth2::basic::BasicTokenResponse,
    oauth2::basic::BasicTokenIntrospectionResponse,
    oauth2::StandardRevocableToken,
    oauth2::basic::BasicRevocationErrorResponse,
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet
>;

struct ClientInfo {
    client_id: ClientId,
    client_secret: ClientSecret,
    auth_url: AuthUrl,
    token_url: TokenUrl,
    redirect_url: RedirectUrl,
    scopes: Vec<Scope>,
    auth_type: AuthType,
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
}

pub struct AuthorizationData {
    pub auth_url: String,
    pub csrf_token: String,
    pub pkce_verifier: String,
}

fn build_oauth_client(client_info: &ClientInfo) -> ConfiguredClient {
    BasicClient::new(client_info.client_id.clone())
        .set_client_secret(client_info.client_secret.clone())
        .set_auth_uri(client_info.auth_url.clone())
        .set_token_uri(client_info.token_url.clone())
        .set_redirect_uri(client_info.redirect_url.clone())
        .set_auth_type(client_info.auth_type.clone())
}

fn generate_auth_url(client_info: &ClientInfo) -> AuthorizationData {
    // This function is just a placeholder to indicate where the auth URL generation would occur
    let client = build_oauth_client(client_info);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(client_info.scopes.clone())
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    AuthorizationData {
        auth_url: auth_url.to_string(),
        csrf_token: csrf_token.secret().to_string(),
        pkce_verifier: pkce_verifier.secret().to_string(),
    }
}

async fn test_oauth(provider: OAuthProvider) -> Result<(), Box<dyn std::error::Error>> {
    // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
    // token URL.
    let client_info = ClientInfo::from_provider(provider);
    let auth_data = generate_auth_url(&client_info);
    let client = build_oauth_client(&client_info);
    let csrf_token = CsrfToken::new(auth_data.csrf_token.clone());
    let pkce_verifier = oauth2::PkceCodeVerifier::new(auth_data.pkce_verifier.clone());

    // This is the URL you should redirect the user to, in order to trigger the authorization
    // process.
    // Respond to the front end with the URL, or open it in a browser.

    // In a real application, you would extract the authorization code from the callback URL
    // For now, you'll need to manually paste the code from the redirect URL
    println!("Please enter the callback URL:");
    let mut callback_url = String::new();
    std::io::stdin().read_line(&mut callback_url)?;
    let url: Url = Url::parse(callback_url.trim())?;
    let auth_code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.to_string())
        .ok_or("No code parameter in callback URL")?;

    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.to_string())
        .ok_or("No state parameter in callback URL")?;

    // Once the user has been redirected to the redirect URL, you'll have access to the
    // authorization code. For security reasons, your code should verify that the `state`
    // parameter returned by the server matches `csrf_token`.
    if &state != csrf_token.secret() {
        return Err("CSRF token mismatch".into());
    } else {
        println!("CSRF token OK");
    }

    // Token requests are made to the token endpoint, using the authorization code previously
    let http_client = reqwest::ClientBuilder
        ::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");
    println!("HTTP client built");
    println!("client id: {}", client.client_id().to_string());

    // Now you can trade it for an access token.
    let token_result = client
        .exchange_code(AuthorizationCode::new(auth_code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(pkce_verifier)
        .request_async(&http_client).await?;

    println!("Access token should be received");
    println!(
        "Tokens: {:?}, {:?}, {:?}",
        token_result.access_token().secret(),
        token_result.refresh_token().map(|t| t.secret()),
        token_result.expires_in()
    );
    // This is where the access token and the refresh token will be stored in the database associated
    // with the user.

    Ok(())
}

pub async fn strava_test_oauth() -> Result<(), Box<dyn std::error::Error>> {
    test_oauth(OAuthProvider::Strava).await
}

pub async fn spotify_test_oauth() -> Result<(), Box<dyn std::error::Error>> {
    test_oauth(OAuthProvider::Spotify).await
}

#[cfg(test)]
mod tests {}
