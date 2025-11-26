use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit};

use crate::crypto::{CryptoError, KEY_SIZE, NONCE_SIZE};

#[derive(Clone)]
pub struct Cipher {
    cipher: Aes256Gcm,
}

impl Cipher {
    #[must_use]
    pub fn new(key: &[u8; KEY_SIZE]) -> Self {
        let cipher = Aes256Gcm::new(key.into());
        Self { cipher }
    }

    /// # Errors
    /// Returns `CryptoError::EncryptionFailed` if encryption fails
    pub fn encrypt(
        &self,
        nonce: &[u8; NONCE_SIZE],
        plaintext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        self.cipher
            .encrypt(nonce.into(), plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))
    }

    /// # Errors
    /// Returns `CryptoError::DecryptionFailed` if decryption fails or authentication tag is invalid
    pub fn decrypt(
        &self,
        nonce: &[u8; NONCE_SIZE],
        ciphertext: &[u8],
    ) -> Result<Vec<u8>, CryptoError> {
        self.cipher
            .decrypt(nonce.into(), ciphertext)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::AUTH_TAG_SIZE;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key_bytes = [0u8; KEY_SIZE];
        let cipher = Cipher::new(&key_bytes);
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, world!";
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertext() {
        let key_bytes = [0u8; KEY_SIZE];
        let cipher = Cipher::new(&key_bytes);
        let nonce1 = [1u8; NONCE_SIZE];
        let nonce2 = [2u8; NONCE_SIZE];
        let plaintext = b"Hello, world!";
        let ciphertext1 = cipher.encrypt(&nonce1, plaintext).unwrap();
        let ciphertext2 = cipher.encrypt(&nonce2, plaintext).unwrap();
        assert_ne!(
            ciphertext1, ciphertext2,
            "Different nonces should produce different ciphertexts"
        );
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key_bytes = [0u8; KEY_SIZE];
        let cipher = Cipher::new(&key_bytes);
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, world!";
        let mut ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        ciphertext[0] ^= 0xff; // Flip the first byte
        let result = cipher.decrypt(&nonce, &ciphertext);
        assert!(
            result.is_err(),
            "Tampered ciphertext should fail decryption"
        );
    }

    #[test]
    fn test_wrong_key_fails() {
        let key_a = [0u8; KEY_SIZE];
        let key_b = [1u8; KEY_SIZE];
        let cipher_a = Cipher::new(&key_a);
        let cipher_b = Cipher::new(&key_b);
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, world!";
        let ciphertext = cipher_a.encrypt(&nonce, plaintext).unwrap();
        let result = cipher_b.decrypt(&nonce, &ciphertext);
        assert!(result.is_err(), "Decrypting with wrong key should fail");
    }

    #[test]
    fn test_ciphertext_length() {
        let key_bytes = [0u8; KEY_SIZE];
        let cipher = Cipher::new(&key_bytes);
        let nonce = [1u8; NONCE_SIZE];
        let plaintext = b"Hello, world!";
        let ciphertext = cipher.encrypt(&nonce, plaintext).unwrap();
        assert_eq!(
            ciphertext.len(),
            plaintext.len() + AUTH_TAG_SIZE,
            "Ciphertext length should be plaintext length + AUTH_TAG_SIZE bytes for auth tag"
        );
    }
}
