#![allow(dead_code)]
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use domain::error::AppError;

const MEMORY_SIZE: u32 = 65536;
const ITERATIONS: u32 = 3;
const PARALLELISM: u32 = 4;
const OUTPUT_LENGTH: usize = 32;

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(MEMORY_SIZE, ITERATIONS, PARALLELISM, Some(OUTPUT_LENGTH))
        .map_err(|e| AppError::InternalError(format!("Argon2 params error: {}", e)))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

    Ok(password_hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::InternalError(format!("Invalid password hash: {}", e)))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(AppError::InternalError(format!(
            "Password verification error: {}",
            e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_success() {
        let password = "my_secure_password";
        let hash = hash_password(password).expect("Failed to hash password");
        assert!(hash.starts_with("$argon2id"));
    }

    #[test]
    fn test_verify_password_success() {
        let password = "my_secure_password";
        let hash = hash_password(password).expect("Failed to hash password");
        let valid = verify_password(password, &hash).expect("Failed to verify password");
        assert!(valid);
    }

    #[test]
    fn test_verify_password_failure() {
        let password = "my_secure_password";
        let hash = hash_password(password).expect("Failed to hash password");
        let valid = verify_password("wrong_password", &hash).expect("Failed to verify password");
        assert!(!valid);
    }
}
