pub mod cipher;
pub mod error;
pub mod key;
pub mod payload;
pub mod service;

pub use cipher::*;
pub use error::*;
pub use key::*;
pub use payload::*;
pub use service::*;

/// AES-256 key size in bytes
pub const KEY_SIZE: usize = 32;
/// AES-GCM nonce size in bytes (96 bits as recommended by NIST)
pub const NONCE_SIZE: usize = 12;
/// AES-GCM authentication tag size in bytes
pub const AUTH_TAG_SIZE: usize = 16;
/// Current payload version
pub const CURRENT_VERSION: u8 = 1;
