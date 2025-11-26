use rand::{rng, RngCore};
use std::path::Path;

use crate::crypto::EncryptedPayload;
use crate::crypto::{Cipher, CryptoError, Key, CURRENT_VERSION, NONCE_SIZE};

#[derive(Clone)]
pub struct EncryptionService {
    cipher: Cipher,
}

impl EncryptionService {
    /// # Errors
    /// Returns `CryptoError` if the encryption key file cannot be loaded
    pub fn from_file(path: &Path) -> Result<Self, CryptoError> {
        let key = Key::from_file(path)?;
        let cipher = Cipher::new(key.as_bytes());
        Ok(Self { cipher })
    }

    /// # Errors
    /// Returns `CryptoError::EncryptionFailed` if encryption fails
    pub fn encrypt(&self, plaintext: &str) -> Result<String, CryptoError> {
        let mut nonce = [0u8; NONCE_SIZE];
        rng().fill_bytes(&mut nonce);

        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes())?;

        let payload = EncryptedPayload {
            version: CURRENT_VERSION,
            nonce,
            ciphertext,
        };

        Ok(payload.to_base64())
    }

    /// # Errors
    /// Returns `CryptoError` if decryption fails or the encrypted string is invalid
    pub fn decrypt(&self, encrypted: &str) -> Result<String, CryptoError> {
        let payload = EncryptedPayload::from_base64(encrypted)?;

        if payload.version != CURRENT_VERSION {
            return Err(CryptoError::UnsupportedVersion(payload.version));
        }

        let plaintext_bytes = self.cipher.decrypt(&payload.nonce, &payload.ciphertext)?;

        String::from_utf8(plaintext_bytes).map_err(|_| CryptoError::InvalidUtf8)
    }
}
