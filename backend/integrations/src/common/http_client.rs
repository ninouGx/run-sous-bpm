pub struct AuthenticatedClient {
    http: reqwest::Client,
}

impl Default for AuthenticatedClient {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthenticatedClient {
    /// Creates a new authenticated HTTP client
    ///
    /// # Panics
    ///
    /// Panics if the HTTP client fails to build (should never happen with default config)
    #[must_use]
    pub fn new() -> Self {
        let http = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        Self { http }
    }

    /// Makes a GET request with Bearer token authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    pub async fn get_with_bearer(
        &self,
        url: &str,
        bearer_token: &str,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http.get(url).bearer_auth(bearer_token).send().await
    }

    /// Makes a GET request with Bearer token authentication and query parameters
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or query serialization fails
    pub async fn get_with_bearer_and_query<Q: serde::Serialize>(
        &self,
        url: &str,
        bearer_token: &str,
        query: &Q,
    ) -> Result<reqwest::Response, reqwest::Error> {
        self.http
            .get(url)
            .bearer_auth(bearer_token)
            .query(query)
            .send()
            .await
    }

    /// Makes a POST request with Bearer token authentication
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails
    pub async fn post_with_bearer(
        &self,
        url: &str,
        bearer_token: &str,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let request = self.http.post(url).bearer_auth(bearer_token);
        let request = if let Some(body) = body {
            request.json(&body)
        } else {
            request
        };
        request.send().await
    }
}
