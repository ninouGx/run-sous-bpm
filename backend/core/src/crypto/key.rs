use hkdf::Hkdf;
use sha2::Sha256;
use std::path::Path;
use zeroize::Zeroize;

use crate::crypto::{CryptoError, CURRENT_VERSION, KEY_SIZE};
const MAX_FILE_SIZE: u64 = 1024; // 1 KB
const HKDF_SALT: &[u8] = b"run-sous-bpm-salt";

pub struct Key {
    bytes: [u8; KEY_SIZE],
}

impl Key {
    /// Load and derive a key from a file
    ///
    /// # Security Checks
    /// - File must exist
    /// - File permissions must be 0o400 (owner read only)
    /// - Passphrase must be at least 32 characters
    ///
    /// # Errors
    /// Returns `CryptoError` if the file cannot be read, has invalid permissions, or key derivation fails
    pub fn from_file(path: &Path) -> Result<Self, CryptoError> {
        let metadata = std::fs::metadata(path).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => CryptoError::KeyFileNotFound(path.to_path_buf()),
            _ => CryptoError::KeyFileReadError(e),
        })?;
        if !metadata.is_file() {
            return Err(CryptoError::KeyFileNotFound(path.to_path_buf()));
        }
        check_file_permissions(path)?;
        if metadata.len() > MAX_FILE_SIZE {
            return Err(CryptoError::KeyFileTooLarge(
                path.to_path_buf(),
                metadata.len(),
            ));
        }
        let contents = std::fs::read_to_string(path).map_err(CryptoError::KeyFileReadError)?;
        let passphrase = contents.trim();
        if passphrase.len() < KEY_SIZE {
            return Err(CryptoError::KeyDerivationFailed);
        }
        // Derive key using HKDF-SHA256
        let bytes = Self::derive_key(passphrase, CURRENT_VERSION, "oauth-tokens")?;
        Ok(Self { bytes })
    }

    fn derive_key(
        passphrase: &str,
        version: u8,
        purpose: &str,
    ) -> Result<[u8; KEY_SIZE], CryptoError> {
        let info = format!("run-sous-bpm-v{version}-{purpose}");
        let hk = Hkdf::<Sha256>::new(Some(HKDF_SALT), passphrase.as_bytes());
        let mut output = [0u8; KEY_SIZE];
        hk.expand(info.as_bytes(), &mut output)
            .map_err(|_| CryptoError::KeyDerivationFailed)?;
        Ok(output)
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8; KEY_SIZE] {
        &self.bytes
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

#[cfg(unix)]
fn check_file_permissions(path: &Path) -> Result<(), CryptoError> {
    use std::os::unix::fs::PermissionsExt;
    let metadata = std::fs::metadata(path).map_err(CryptoError::KeyFileReadError)?;
    let permissions = metadata.permissions();
    let mode = permissions.mode();
    if (mode & 0o777) != 0o400 {
        return Err(CryptoError::KeyFilePermissionsTooOpen(
            path.to_path_buf(),
            mode,
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn check_file_permissions(_path: &Path) -> Result<(), CryptoError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_derivation_consistency() {
        let purpose = "oauth-tokens";
        let passphrase = "my-super-secret-passphrase-at-least-32-chars";
        let key1 = Key::derive_key(passphrase, 1, purpose).unwrap();
        let key2 = Key::derive_key(passphrase, 1, purpose).unwrap();
        assert_eq!(key1, key2, "Same passphrase should derive same key");
    }

    #[test]
    fn test_version_separation() {
        let purpose = "oauth-tokens";
        let passphrase = "my-super-secret-passphrase-at-least-32-chars";
        let key_v1 = Key::derive_key(passphrase, 1, purpose).unwrap();
        let key_v2 = Key::derive_key(passphrase, 2, purpose).unwrap();
        assert_ne!(
            key_v1, key_v2,
            "Different versions should derive different keys"
        );
    }
}
