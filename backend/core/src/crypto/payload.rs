use base64::{engine::general_purpose::STANDARD_NO_PAD, Engine};

use crate::crypto::{CryptoError, NONCE_SIZE};

/// Minimum payload size: 1 byte version + `NONCE_SIZE` bytes nonce
const MIN_PAYLOAD_SIZE: usize = 1 + NONCE_SIZE;

pub struct EncryptedPayload {
    pub(crate) version: u8,
    pub(crate) nonce: [u8; NONCE_SIZE],
    pub(crate) ciphertext: Vec<u8>,
}

// Implement serialization and deserialization for EncryptedPayload that can be stored in the database as a base64 string.
impl EncryptedPayload {
    #[must_use]
    pub fn to_base64(&self) -> String {
        let mut bytes = Vec::with_capacity(MIN_PAYLOAD_SIZE + self.ciphertext.len());
        bytes.push(self.version);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.ciphertext);
        STANDARD_NO_PAD.encode(&bytes)
    }

    /// # Errors
    /// Returns `CryptoError` if the base64 string is invalid or the payload format is incorrect
    pub fn from_base64(s: &str) -> Result<Self, CryptoError> {
        let bytes = STANDARD_NO_PAD.decode(s)?;
        if bytes.len() < MIN_PAYLOAD_SIZE {
            return Err(CryptoError::InvalidPayloadFormat(
                "Payload too short".to_string(),
            ));
        }
        let version = bytes[0];
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(&bytes[1..MIN_PAYLOAD_SIZE]);
        let ciphertext = bytes[MIN_PAYLOAD_SIZE..].to_vec();
        Ok(Self {
            version,
            nonce,
            ciphertext,
        })
    }
}
