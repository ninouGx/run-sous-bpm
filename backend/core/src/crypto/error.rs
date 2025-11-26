use std::fmt::{self, Display};

/// Errors that can occur during encryption/decryption operations
#[derive(Debug)]
pub enum CryptoError {
    // Key loading errors (startup)
    KeyFileNotFound(std::path::PathBuf),
    KeyFilePermissionsTooOpen(std::path::PathBuf, u32), // path, mode
    KeyFileReadError(std::io::Error),
    KeyFileTooLarge(std::path::PathBuf, u64), // path, size
    KeyDerivationFailed,

    // Encryption errors (runtime)
    EncryptionFailed(String), // Include error details

    // Decryption errors (runtime)
    InvalidBase64(base64::DecodeError),
    InvalidPayloadFormat(String), // What was wrong
    InvalidUtf8,
    UnsupportedVersion(u8), // Which version we saw
    DecryptionFailed(String),
}

impl Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::KeyFileNotFound(path) => {
                write!(f, "Key file not found: {}", path.display())
            }
            CryptoError::KeyFilePermissionsTooOpen(path, mode) => write!(
                f,
                "Key file permissions too open (mode {:o}): {}",
                mode,
                path.display()
            ),
            CryptoError::KeyFileReadError(err) => write!(f, "Failed to read key file: {err}"),
            CryptoError::KeyFileTooLarge(path, size) => {
                write!(f, "Key file too large ({size} bytes): {}", path.display())
            }
            CryptoError::KeyDerivationFailed => write!(f, "Key derivation failed"),
            CryptoError::EncryptionFailed(details) => {
                write!(f, "Encryption failed: {details}")
            }
            CryptoError::InvalidBase64(err) => write!(f, "Invalid Base64 encoding: {err}"),
            CryptoError::InvalidPayloadFormat(details) => {
                write!(f, "Invalid payload format: {details}")
            }
            CryptoError::InvalidUtf8 => write!(f, "Decrypted data is not valid UTF-8"),
            CryptoError::UnsupportedVersion(version) => {
                write!(f, "Unsupported version: {version}")
            }
            CryptoError::DecryptionFailed(details) => {
                write!(f, "Decryption failed: {details}")
            }
        }
    }
}

impl std::error::Error for CryptoError {}

impl From<base64::DecodeError> for CryptoError {
    fn from(err: base64::DecodeError) -> Self {
        CryptoError::InvalidBase64(err)
    }
}
