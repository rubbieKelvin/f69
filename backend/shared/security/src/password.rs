//! Argon2 password hashing for stored credentials.

use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;

/// Password hashing failures (invalid params or backend errors).
#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("hashing failed: {0}")]
    Hash(argon2::password_hash::Error),
}

/// Returns a PHC string suitable for persisting on a user record.
pub fn hash_password(plain: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let argon2 = Argon2::default();
    argon2
        .hash_password(plain.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(PasswordError::Hash)
}

/// Constant-time verification against a PHC hash string.
pub fn verify_password(hash: &str, plain: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(plain.as_bytes(), &parsed)
        .is_ok()
}
