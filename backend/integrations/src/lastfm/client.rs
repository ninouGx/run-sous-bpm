use lastfm_client::LastFmClient as LastFmApiClient;
use lastfm_client::types::RecentTrack;

use crate::common::IntegrationError;

/// Last.fm API client for fetching user listening history
pub struct LastFmClient {
    client: LastFmApiClient,
}

impl Default for LastFmClient {
    fn default() -> Self {
        Self::new()
    }
}

impl LastFmClient {
    /// Creates a new Last.fm API client
    ///
    /// Reads the API key from the `LAST_FM_API_KEY` environment variable.
    ///
    /// # Panics
    /// Panics if the `LAST_FM_API_KEY` environment variable is not set
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: LastFmApiClient::new()
                .expect("LAST_FM_API_KEY environment variable must be set"),
        }
    }

    /// Fetches tracks played within a specific time range
    ///
    /// # Arguments
    /// * `username` - Last.fm username to fetch data for
    /// * `start_timestamp` - Unix timestamp (seconds) for start of range
    /// * `end_timestamp` - Unix timestamp (seconds) for end of range
    ///
    /// # Errors
    ///
    /// Returns an error if the Last.fm API request fails
    ///
    /// # Returns
    /// Vector of `RecentTrack` sorted chronologically, filtered to exclude "now playing" tracks
    ///
    /// # Note
    /// Uses Last.fm API's native `from` and `to` parameters for efficient server-side filtering
    pub async fn get_tracks_in_time_range(
        &self,
        username: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<Vec<RecentTrack>, IntegrationError> {
        // Fetch tracks between timestamps using Last.fm API's native time range filtering
        let tracks = self
            .client
            .recent_tracks(username)
            .between(start_timestamp, end_timestamp)
            .fetch()
            .await
            .map_err(|e| IntegrationError::Other(e.to_string()))?;

        // Filter out "now playing" tracks (tracks without a timestamp)
        let filtered_tracks: Vec<RecentTrack> = tracks
            .into_iter()
            .filter(|track| track.date.is_some())
            .collect();

        Ok(filtered_tracks)
    }

    /// Fetches the most recent N tracks for a user
    ///
    /// # Arguments
    /// * `username` - Last.fm username to fetch data for
    /// * `limit` - Number of recent tracks to fetch
    ///
    /// # Errors
    ///
    /// Returns an error if the Last.fm API request fails
    pub async fn get_recent_tracks(
        &self,
        username: &str,
        limit: u32,
    ) -> Result<Vec<RecentTrack>, IntegrationError> {
        self.client
            .recent_tracks(username)
            .limit(limit)
            .fetch()
            .await
            .map_err(|e| IntegrationError::Other(e.to_string()))
    }
}
