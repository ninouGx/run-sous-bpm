use argon2::{
    password_hash::{rand_core::OsRng, Error, SaltString},
    Argon2,
};

/// Hashes a password using Argon2id algorithm with random salt
///
/// # Errors
///
/// Returns an error if password hashing fails due to invalid configuration or salt generation issues
pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let config = Argon2::default();
    let password_hash =
        argon2::PasswordHasher::hash_password(&config, password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

/// Verifies a password against a stored Argon2 hash
///
/// # Errors
///
/// Returns an error if:
/// - Password hash string is malformed or invalid
/// - Verification process fails for reasons other than password mismatch
pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, Error> {
    let parsed_hash = argon2::PasswordHash::new(password_hash)?;
    let config = Argon2::default();
    match argon2::PasswordVerifier::verify_password(&config, password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}
