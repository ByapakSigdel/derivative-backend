//! Password hashing and verification utilities using Argon2.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use tracing::{debug, error};

use crate::errors::{AppError, AppResult};

/// Default Argon2 parameters for password hashing.
/// These are tuned for security while maintaining reasonable performance.
fn argon2_params() -> Argon2<'static> {
    // Using Argon2id with strong parameters
    // - Memory: 64 MiB
    // - Iterations: 3
    // - Parallelism: 4
    let params = Params::new(
        64 * 1024, // 64 MiB memory cost
        3,         // 3 iterations
        4,         // 4 lanes of parallelism
        None,      // Default output length (32 bytes)
    )
    .expect("Invalid Argon2 parameters");

    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

/// Hash a password using Argon2id.
///
/// # Arguments
/// * `password` - The plaintext password to hash
///
/// # Returns
/// The PHC-formatted password hash string
pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = argon2_params();

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to hash password: {}", e)))?;

    Ok(password_hash.to_string())
}

/// Verify a password against a stored hash.
///
/// # Arguments
/// * `password` - The plaintext password to verify
/// * `hash` - The stored password hash to verify against
///
/// # Returns
/// `true` if the password matches, `false` otherwise
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    debug!(
        "Verifying password against hash (hash length: {})",
        hash.len()
    );

    let parsed_hash = match PasswordHash::new(hash) {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to parse password hash: {}. Hash value: {}", e, hash);
            return Err(AppError::Internal(anyhow::anyhow!(
                "Invalid password hash format in database. The stored hash may be corrupted or invalid. Error: {}",
                e
            )));
        }
    };

    let argon2 = argon2_params();

    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            debug!("Password verification succeeded");
            Ok(true)
        }
        Err(argon2::password_hash::Error::Password) => {
            debug!("Password verification failed: incorrect password");
            Ok(false)
        }
        Err(e) => {
            error!("Password verification error: {}", e);
            Err(AppError::Internal(anyhow::anyhow!(
                "Password verification failed: {}",
                e
            )))
        }
    }
}

/// Validate password strength requirements.
///
/// # Requirements
/// - Minimum 8 characters
/// - Maximum 128 characters
///
/// # Returns
/// `Ok(())` if valid, `Err(AppError::Validation)` if invalid
pub fn validate_password_strength(password: &str) -> AppResult<()> {
    if password.len() < 8 {
        return Err(AppError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    if password.len() > 128 {
        return Err(AppError::Validation(
            "Password must be at most 128 characters long".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "secure_password_123!";
        let hash = hash_password(password).unwrap();

        // Verify correct password
        assert!(verify_password(password, &hash).unwrap());

        // Verify incorrect password
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let password1 = "password1";
        let password2 = "password2";

        let hash1 = hash_password(password1).unwrap();
        let hash2 = hash_password(password2).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_same_password_different_hashes() {
        let password = "same_password";

        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        // Each hash should be unique due to random salt
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_password_strength_validation() {
        // Too short
        assert!(validate_password_strength("short").is_err());

        // Valid length
        assert!(validate_password_strength("validpassword").is_ok());

        // Exactly minimum length
        assert!(validate_password_strength("12345678").is_ok());

        // Too long (over 128 chars)
        let long_password: String = "a".repeat(129);
        assert!(validate_password_strength(&long_password).is_err());
    }
}
