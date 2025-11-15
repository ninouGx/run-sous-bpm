pub mod client;

pub use client::*;

use serde::{Deserialize, Serialize};

/// Spotify API response types for music enrichment pipeline
///
/// These types are foundation for future implementation that will:
/// - Fetch recently played tracks from Spotify
/// - Enrich Last.fm scrobbles with Spotify metadata
/// - Add audio features (tempo, energy, etc.) for performance analytics
#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyRecentlyPlayedResponse {
    pub total: u32,
    pub items: Vec<SpotifyPlayedItem>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyPlayedItem {
    pub played_at: String,
    pub track: SpotifyTrack,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub external_urls: SpotifyExternalUrls,
    pub duration_ms: u32,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyAlbum {
    pub id: String,
    pub name: String,
    pub images: Vec<SpotifyImage>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyImage {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyExternalUrls {
    pub spotify: String,
}
