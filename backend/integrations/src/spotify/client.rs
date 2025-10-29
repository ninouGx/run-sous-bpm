use serde::{ Deserialize, Serialize };

use crate::{ common::{ IntegrationClient, IntegrationError } };

pub struct SpotifyRecentlyPlayedParams {
    pub after: Option<u64>,
    pub before: Option<u64>,
}

pub struct SpotifyApiClient {
    pub integration_client: IntegrationClient,
    pub base_url: String,
}

impl SpotifyApiClient {
    /// Creates a new Spotify API client
    #[must_use]
    pub fn new(integration_client: IntegrationClient, base_url: String) -> Self {
        Self {
            integration_client,
            base_url,
        }
    }
    /// Fetches the recently played tracks of the authenticated user from Spotify
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response deserialization fails
    pub async fn get_recently_played_tracks(
        &self,
        access_token: &str,
        param: SpotifyRecentlyPlayedParams
    ) -> Result<SpotifyRecentlyPlayedResponse, IntegrationError> {
        let mut url = format!("{self.base_url}/me/player/recently-played");

        let response = self.integration_client.get_with_query(&url, access_token, &param).await?;
        let spotify_response: SpotifyRecentlyPlayedResponse = response
            .json().await
            .map_err(IntegrationError::from)?;
        Ok(spotify_response)
    }
}
