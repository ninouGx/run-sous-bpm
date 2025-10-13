use std::time::Duration;
use moka::sync::Cache;

use crate::config::OAuthProvider;

#[derive(Clone)]
pub struct OAuthState {
    pub pkce_verifier: String,
    pub provider: OAuthProvider,
}

pub struct OAuthSessionManager {
    cache: Cache<String, OAuthState>,
}

impl OAuthSessionManager {
    pub fn new() -> Self {
        let cache = Cache::builder()
            .time_to_live(Duration::from_secs(600)) // 10 minutes TTL
            .max_capacity(1000) // Max 1000 sessions
            .build();

        Self { cache }
    }

    pub fn store(&self, csrf_token: String, state: OAuthState) {
        self.cache.insert(csrf_token, state);
    }

    pub fn consume(&self, csrf_token: &str) -> Option<OAuthState> {
        self.cache.remove(csrf_token)
    }
}
