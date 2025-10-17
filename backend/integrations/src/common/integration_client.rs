use std::sync::Arc;

use reqwest::Response;

use crate::common::{AuthenticatedClient, IntegrationError};

/// HTTP client for integration APIs with OAuth token management
///
/// Wraps the authenticated HTTP client and provides convenience methods
/// for making API requests with Bearer token authentication.
pub struct IntegrationClient {
    pub http_client: Arc<AuthenticatedClient>,
}

impl IntegrationClient {
    /// Creates a new integration client with the provided HTTP client
    #[must_use]
    pub fn new(http_client: Arc<AuthenticatedClient>) -> Self {
        Self { http_client }
    }

    /// Makes a GET request with OAuth Bearer token authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    pub async fn get(&self, url: &str, access_token: &str) -> Result<Response, IntegrationError> {
        let response = self.http_client.get_with_bearer(url, access_token).await?;
        Ok(response)
    }

    /// Makes a GET request with OAuth Bearer token authentication and query parameters
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or query serialization fails
    pub async fn get_with_query<Q: serde::Serialize>(
        &self,
        url: &str,
        access_token: &str,
        query: &Q,
    ) -> Result<Response, IntegrationError> {
        let response = self
            .http_client
            .get_with_bearer_and_query(url, access_token, query)
            .await?;
        Ok(response)
    }
}
