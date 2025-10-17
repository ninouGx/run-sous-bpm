// Strava API integration will be implemented here
pub mod client;

pub use client::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug)]
pub struct StravaActivityResponse {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub sport_type: String,
    pub start_date: String,
    pub moving_time: i32,
    pub elapsed_time: i32,
    pub timezone: String,
    pub distance: f32,
    pub total_elevation_gain: f32,
}

/*
{
  "altitude": {
    "data": [245.2, 247.1, 249.8, ...],
    "original_size": 1327
  },
  "distance": {
    "data": [0.0, 5.2, 10.8, ...],
    "original_size": 1327
  },
  "latlng": {
    "data": [[48.8566, 2.3522], [48.8567, 2.3521], ...],
    "original_size": 1327
  },
  "time": {
    "data": [0, 1, 2, 3, ...],
    "original_size": 1327
  },
  "velocity_smooth": {
    "data": [0.0, 3.2, 3.5, ...],
    "original_size": 1327
  }
} */
#[derive(Deserialize, Serialize, Debug)]
pub struct StravaActivityStreamResponse(pub HashMap<String, StreamData>);

#[derive(Deserialize, Serialize, Debug)]
pub struct StreamData {
    pub data: Vec<serde_json::Value>,
    pub original_size: u32,
}
