use serde::Serialize;

use crate::common::{IntegrationClient, IntegrationError};
use crate::spotify::SpotifyRecentlyPlayedResponse;

/// Query parameters for Spotify recently played endpoint
///
/// This will be used in the future music enrichment pipeline
#[allow(dead_code)]
#[derive(Serialize)]
pub struct SpotifyRecentlyPlayedParams {
    pub after: Option<u64>,
    pub before: Option<u64>,
}

/// Spotify API client for music enrichment
///
/// Foundation for future Spotify integration to enrich Last.fm data
/// with audio features (tempo, energy, danceability, etc.)
#[allow(dead_code)]
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
        let url = format!("{}/me/player/recently-played", self.base_url);

        let response = self.integration_client.get_with_query(&url, access_token, &param).await?;
        let spotify_response: SpotifyRecentlyPlayedResponse = response
            .json().await
            .map_err(IntegrationError::from)?;
        Ok(spotify_response)
    }
}
