use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, Display, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OAuthProvider {
    Strava,
    Spotify,
}

/*pub mod user;
pub mod workout;
pub mod music;

pub use user::*;
pub use workout::*;
pub use music::*;*/
