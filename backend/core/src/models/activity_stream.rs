use std::collections::HashMap;

use run_sous_bpm_integrations::strava::{StravaActivityStreamResponse, StreamData};
use sea_orm::{ActiveValue::Set, prelude::DateTimeWithTimeZone};
use tracing::info;
use uuid::Uuid;

use crate::database::activity_stream;

/// DTO for creating an activity stream from Strava API response
#[derive(Debug, Clone)]
pub struct ValidatedActivityStreams {
    pub activity_id: Uuid,
    pub time: Vec<f32>,
    pub distance: Vec<f32>,
    pub latlng: Option<Vec<(f32, f32)>>,
    pub altitude: Option<Vec<f32>>,
    pub heart_rate: Option<Vec<i32>>,
    pub cadence: Option<Vec<i32>>,
    pub watts: Option<Vec<f32>>,
    pub velocity: Option<Vec<f32>>,
    pub temperature: Option<Vec<f32>>,
}

impl ValidatedActivityStreams {
    /// Creates a DTO from Strava API response
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `start_date` cannot be parsed as ISO 8601 datetime
    /// - `external_id` cannot be converted to i64
    #[allow(clippy::needless_pass_by_value)]
    pub fn from_strava_response(
        response: StravaActivityStreamResponse,
        activity_id: Uuid,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!(
            activity_id = %activity_id,
            "Validating Strava activity streams for activity"
        );
        let time = extract_required_f32(&response.0, "time")?;
        let distance = extract_required_f32(&response.0, "distance")?;
        info!(
            activity_id = %activity_id,
            points = time.len(),
            "Validated {} activity stream points",
            time.len()
        );
        #[allow(clippy::cast_possible_truncation)]
        let latlng = if let Some(stream) = response.0.get("latlng") {
            let latlng_data = stream
                .data
                .iter()
                .filter_map(|v| {
                    if let Some(arr) = v.as_array() {
                        if arr.len() == 2 {
                            let lat = arr[0].as_f64()? as f32;
                            let lng = arr[1].as_f64()? as f32;
                            Some((lat, lng))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            Some(latlng_data)
        } else {
            None
        };
        let altitude = extract_optional_f32(&response.0, "altitude");
        let heart_rate = extract_optional_i32(&response.0, "heart_rate");
        let cadence = extract_optional_i32(&response.0, "cadence");
        let watts = extract_optional_f32(&response.0, "watts");
        let velocity = extract_optional_f32(&response.0, "velocity_smooth");
        let temperature = extract_optional_f32(&response.0, "temperature");

        let lengths = [
            time.len(),
            distance.len(),
            latlng.as_ref().map_or(0, Vec::len),
            altitude.as_ref().map_or(time.len(), Vec::len),
            heart_rate.as_ref().map_or(time.len(), Vec::len),
            cadence.as_ref().map_or(time.len(), Vec::len),
            watts.as_ref().map_or(time.len(), Vec::len),
            velocity.as_ref().map_or(time.len(), Vec::len),
            temperature.as_ref().map_or(time.len(), Vec::len),
        ];
        if !lengths.iter().all(|&len| len == time.len()) {
            return Err("Inconsistent stream data lengths".into());
        }

        Ok(Self {
            activity_id,
            time,
            distance,
            latlng,
            altitude,
            heart_rate,
            cadence,
            watts,
            velocity,
            temperature,
        })
    }

    /// Converts the DTO into a `SeaORM` `ActiveModel` for insertion
    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    pub fn into_active_models(
        self,
        start_time: DateTimeWithTimeZone,
    ) -> Vec<activity_stream::ActiveModel> {
        let len = self.time.len();
        let mut models = Vec::with_capacity(len);
        info!(
            activity_id = %self.activity_id,
            points = len,
            "Preparing {} activity stream points for insertion",
            len
        );
        for i in 0..len {
            models.push(activity_stream::ActiveModel {
                activity_id: Set(self.activity_id),
                time: Set(start_time + chrono::Duration::seconds(self.time[i] as i64)),
                distance: Set(Some(self.distance[i])),
                latitude: Set(self
                    .latlng
                    .as_ref()
                    .and_then(|ll| ll.get(i).map(|(lat, _)| f64::from(*lat)))),
                longitude: Set(self
                    .latlng
                    .as_ref()
                    .and_then(|ll| ll.get(i).map(|(_, lng)| f64::from(*lng)))),
                altitude: Set(self.altitude.as_ref().and_then(|alt| alt.get(i).copied())),
                heart_rate: Set(self.heart_rate.as_ref().and_then(|hr| hr.get(i).copied())),
                cadence: Set(self.cadence.as_ref().and_then(|cad| cad.get(i).copied())),
                watts: Set(self.watts.as_ref().and_then(|w| w.get(i).copied())),
                velocity: Set(self.velocity.as_ref().and_then(|v| v.get(i).copied())),
                temperature: Set(self.temperature.as_ref().and_then(|t| t.get(i).copied())),
            });
        }
        models
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn extract_required_f32(map: &HashMap<String, StreamData>, key: &str) -> Result<Vec<f32>, String> {
    if let Some(stream) = map.get(key) {
        Ok(stream
            .data
            .iter()
            .filter_map(|v| {
                // Try as float first, then as signed integer, then as unsigned integer
                v.as_f64()
                    .map(|f| f as f32)
                    .or_else(|| v.as_i64().map(|i| i as f32))
                    .or_else(|| v.as_u64().map(|u| u as f32))
            })
            .collect::<Vec<_>>())
    } else {
        Err(format!("Missing required stream data: {key}"))
    }
}

#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn extract_optional_f32(map: &HashMap<String, StreamData>, key: &str) -> Option<Vec<f32>> {
    map.get(key).map(|stream| {
        stream
            .data
            .iter()
            .filter_map(|v| {
                // Try as float first, then as signed integer, then as unsigned integer
                v.as_f64()
                    .map(|f| f as f32)
                    .or_else(|| v.as_i64().map(|i| i as f32))
                    .or_else(|| v.as_u64().map(|u| u as f32))
            })
            .collect::<Vec<_>>()
    })
}

#[allow(clippy::cast_possible_truncation)]
fn extract_optional_i32(map: &HashMap<String, StreamData>, key: &str) -> Option<Vec<i32>> {
    map.get(key).map(|stream| {
        stream
            .data
            .iter()
            .filter_map(|v| v.as_i64().map(|i| i as i32))
            .collect::<Vec<_>>()
    })
}
