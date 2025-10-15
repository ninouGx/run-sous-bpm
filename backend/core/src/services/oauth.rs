use super::oauth_session::OAuthSessionManager;
use axum_login::tracing::info;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::reqwest;
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, TokenResponse};
use sea_orm::DatabaseConnection;

use crate::config::{ClientInfo, OAuthProvider};
use crate::database::repositories::oauth_token_repository::upsert_oauth_token;
use crate::services::OAuthState;

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
    oauth2::EndpointSet,
>;

pub struct AuthorizationData {
    pub auth_url: String,
}

fn build_oauth_client(client_info: &ClientInfo) -> ConfiguredClient {
    BasicClient::new(client_info.client_id.clone())
        .set_client_secret(client_info.client_secret.clone())
        .set_auth_uri(client_info.auth_url.clone())
        .set_token_uri(client_info.token_url.clone())
        .set_redirect_uri(client_info.redirect_url.clone())
        .set_auth_type(client_info.auth_type.clone())
}

#[must_use]
pub fn start_oauth_flow(
    provider: OAuthProvider,
    session_store: &OAuthSessionManager,
    user_id: uuid::Uuid,
) -> String {
    let client_info = ClientInfo::from_provider(provider);
    let client = build_oauth_client(&client_info);

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(client_info.scopes.clone())
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    let state = OAuthState {
        pkce_verifier: pkce_verifier.secret().to_string(),
        provider,
        user_id,
    };
    session_store.store(csrf_token.secret().to_string(), state);
    auth_url.to_string()
}

/// Handles OAuth callback by exchanging authorization code for access token
///
/// # Errors
///
/// Returns an error if:
/// - CSRF token is invalid or expired
/// - HTTP client fails to build
/// - Token exchange request fails
/// - Database token storage fails
///
/// # Panics
///
/// Panics if:
/// - HTTP client builder fails (should never happen with default config)
/// - Duration conversion from token response fails
pub async fn handle_oauth_callback(
    code: String,
    state: String,
    session_store: &OAuthSessionManager,
    db_connection: &DatabaseConnection,
) -> Result<BasicTokenResponse, Box<dyn std::error::Error>> {
    info!(
        "Handling OAuth callback with code: {}, state: {}",
        code, state
    );
    let Some(session_state) = session_store.consume(&state) else {
        println!("Invalid or expired CSRF token");
        return Err("Invalid or expired CSRF token".into());
    };

    let client_info = ClientInfo::from_provider(session_state.provider);
    let oauth_client = build_oauth_client(&client_info);

    let http_client = reqwest::ClientBuilder::new()
        // Following redirects opens the client up to SSRF vulnerabilities.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result = oauth_client
        .exchange_code(AuthorizationCode::new(code))
        // Set the PKCE code verifier.
        .set_pkce_verifier(PkceCodeVerifier::new(session_state.pkce_verifier))
        .request_async(&http_client)
        .await?;

    upsert_oauth_token(
        db_connection,
        // You would typically get the user ID from your session or context.
        session_state.user_id,
        session_state.provider,
        token_result.access_token().secret().to_string(),
        token_result.refresh_token().map(|r| r.secret().to_string()),
        token_result.expires_in().map(|dur| {
            let expiry = chrono::Utc::now() + chrono::Duration::from_std(dur).unwrap();
            expiry.into()
        }),
        Some(
            client_info
                .scopes
                .iter()
                .map(|s| s.as_ref().to_string())
                .collect(),
        ),
    )
    .await?;

    Ok(token_result)
}

#[cfg(test)]
mod tests {}
