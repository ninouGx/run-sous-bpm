use super::oauth_session::OAuthSessionManager;
use axum_login::tracing::info;
use oauth2::basic::{BasicClient, BasicTokenResponse};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, TokenResponse};
use oauth2::{RefreshToken, reqwest};
use sea_orm::DatabaseConnection;

use crate::config::{ClientInfo, OAuthProvider};
use crate::database::repositories::oauth_token_repository::upsert_oauth_token;
use crate::database::{get_oauth_token_by_provider, oauth_token};
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
) -> Result<(BasicTokenResponse, OAuthProvider), Box<dyn std::error::Error>> {
    info!(
        "Handling OAuth callback with code: {}, state: {}",
        code, state
    );
    let Some(session_state) = session_store.consume(&state) else {
        tracing::error!("Invalid or expired CSRF token");
        return Err("Invalid or expired CSRF token".into());
    };

    let provider = session_state.provider;
    let client_info = ClientInfo::from_provider(provider);
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
        provider,
        token_result.access_token().secret().to_string(),
        token_result.refresh_token().map(|r| r.secret().to_string()),
        token_result.expires_in().map(|dur| {
            let expiry = chrono::Utc::now() + chrono::Duration::from_std(dur)
                .expect("Token expiry duration out of range");
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

    Ok((token_result, provider))
}

/// Gets a valid OAuth access token for a user and provider
///
/// If the token is expired, it will automatically refresh it
///
/// # Errors
///
/// Returns an error if:
/// - Token not found in database
/// - Token refresh fails
/// - Database operation fails
pub async fn get_valid_token(
    db_connection: &DatabaseConnection,
    user_id: uuid::Uuid,
    provider: OAuthProvider,
) -> Result<String, Box<dyn std::error::Error>> {
    let token = get_oauth_token_by_provider(db_connection, user_id, provider).await?;

    let token = token.ok_or("OAuth token not found for user and provider")?;

    if let Some(expires_at) = token.expires_at {
        if expires_at < chrono::Utc::now() {
            if token.refresh_token.is_some() {
                return refresh_token(db_connection, &token, provider).await;
            }
            // Token expired and no refresh token available
            return Err("Token expired and no refresh token available".into());
        }
    }

    Ok(token.access_token)
}

/// Refreshes an expired OAuth token
///
/// # Errors
///
/// Returns an error if:
/// - OAuth provider configuration is missing
/// - HTTP client fails to build
/// - Token exchange request fails
/// - Database update fails
///
/// # Panics
///
/// Panics if the HTTP client fails to build (should never happen with default config)
pub async fn refresh_token(
    db_connection: &DatabaseConnection,
    token: &oauth_token::Model,
    provider: OAuthProvider,
) -> Result<String, Box<dyn std::error::Error>> {
    let refresh_token_str = token
        .refresh_token
        .as_ref()
        .ok_or("No refresh token available")?;
    let refresh_token = RefreshToken::new(refresh_token_str.clone());

    let client_info = ClientInfo::from_provider(provider);
    let oauth_client = build_oauth_client(&client_info);
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token_result = oauth_client
        .exchange_refresh_token(&refresh_token)
        .request_async(&http_client)
        .await
        .map_err(|e| format!("Token refresh request failed: {e}"))?;

    upsert_oauth_token(
        db_connection,
        token.user_id,
        provider,
        token_result.access_token().secret().to_string(),
        token_result.refresh_token().map(|r| r.secret().to_string()),
        token_result.expires_in().map(|dur| {
            let expiry = chrono::Utc::now() + chrono::Duration::from_std(dur)
                .expect("Token expiry duration out of range");
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

    Ok(token_result.access_token().secret().to_string())
}

/// Checks if a user has connected an OAuth provider
///
/// # Errors
///
/// Returns an error if database operation fails
///
/// # Returns
/// True if the user has a token for the provider, false otherwise
pub async fn is_oauth_provider_connected(
    db_connection: &DatabaseConnection,
    user_id: uuid::Uuid,
    provider: OAuthProvider,
) -> Result<bool, Box<dyn std::error::Error>> {
    let token = get_oauth_token_by_provider(db_connection, user_id, provider).await?;
    Ok(token.is_some())
}

#[cfg(test)]
mod tests {}
