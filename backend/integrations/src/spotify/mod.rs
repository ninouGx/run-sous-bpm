pub mod client;

pub use client::*;

use serde::{ Deserialize, Serialize };

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyRecentlyPlayedResponse {
    pub total: u32,
    pub items: Vec<SpotifyPlayedItem>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyPlayedItem {
    pub played_at: String,
    pub track: SpotifyTrack,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub artists: Vec<SpotifyArtist>,
    pub album: SpotifyAlbum,
    pub external_urls: SpotifyExternalUrls,
    pub duration_ms: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyArtist {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyAlbum {
    pub id: String,
    pub name: String,
    pub images: Vec<SpotifyImage>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyImage {
    pub url: String,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SpotifyExternalUrls {
    pub spotify: String,
}
