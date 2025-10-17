use serde::{Deserialize, Serialize};

use crate::{
    common::{IntegrationClient, IntegrationError},
    strava::{StravaActivityResponse, StravaActivityStreamResponse},
};

#[derive(Deserialize, Serialize, Debug)]
pub struct StravaActivitiesParams {
    pub before: Option<u64>,
    pub after: Option<u64>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct StravaActivityStreamsParams {
    pub keys: String,
}
impl StravaActivityStreamsParams {
    /// Creates new activity streams parameters from a list of stream types
    #[must_use]
    pub fn new(keys: &[&str]) -> Self {
        Self {
            keys: keys.join(","),
        }
    }
}

pub struct StravaApiClient {
    pub integration_client: IntegrationClient,
    pub base_url: String,
}

impl StravaApiClient {
    /// Creates a new Strava API client
    #[must_use]
    pub fn new(integration_client: IntegrationClient, base_url: String) -> Self {
        Self {
            integration_client,
            base_url,
        }
    }

    /// Fetches the authenticated athlete's activities from Strava
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response deserialization fails
    pub async fn get_athlete_activities(
        &self,
        access_token: &str,
        query: Option<StravaActivitiesParams>,
    ) -> Result<Vec<StravaActivityResponse>, IntegrationError> {
        let url = format!("{}/athlete/activities", self.base_url);
        let response = self
            .integration_client
            .get_with_query(&url, access_token, &query)
            .await?;
        response
            .json::<Vec<StravaActivityResponse>>()
            .await
            .map_err(|e| IntegrationError::Deserialization(e.to_string()))
    }

    /// Fetches detailed information for a specific activity
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response deserialization fails
    pub async fn get_activity_details(
        &self,
        access_token: &str,
        external_id: i64,
    ) -> Result<StravaActivityResponse, IntegrationError> {
        let url = format!("{}/activities/{}", self.base_url, external_id);
        let response = self.integration_client.get(&url, access_token).await?;
        response
            .json::<StravaActivityResponse>()
            .await
            .map_err(|e| IntegrationError::Deserialization(e.to_string()))
    }

    /// Fetches activity stream data (GPS coordinates, heart rate, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or response deserialization fails
    pub async fn get_activity_streams(
        &self,
        access_token: &str,
        external_id: i64,
        params: StravaActivityStreamsParams,
    ) -> Result<StravaActivityStreamResponse, IntegrationError> {
        let url = format!("{}/activities/{}/streams", self.base_url, external_id);
        let query = serde_json::json!({
            "keys": params.keys,
            "key_by_type": true,
            "series_type": "distance"
        });
        let response = self
            .integration_client
            .get_with_query(&url, access_token, &query)
            .await?;
        response
            .json::<StravaActivityStreamResponse>()
            .await
            .map_err(|e| IntegrationError::Deserialization(e.to_string()))
    }
}
