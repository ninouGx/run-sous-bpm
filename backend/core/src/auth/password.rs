use argon2::{
    Argon2,
    password_hash::{Error, SaltString, rand_core::OsRng},
};

pub fn hash_password(password: &str) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let config = Argon2::default();
    let password_hash =
        argon2::PasswordHasher::hash_password(&config, password.as_bytes(), &salt)?.to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, Error> {
    let parsed_hash = argon2::PasswordHash::new(password_hash)?;
    let config = Argon2::default();
    match argon2::PasswordVerifier::verify_password(&config, password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e),
    }
}
