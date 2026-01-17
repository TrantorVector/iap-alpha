use core::error::AppError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[allow(dead_code)]
pub struct ApiError(pub AppError);

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppState {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl JwtService {
    pub fn new(encoding_key_pem: &[u8], decoding_key_pem: &[u8]) -> Result<Self, AppError> {
        let encoding_key = EncodingKey::from_rsa_pem(encoding_key_pem)
            .map_err(|e| AppError::AuthError(format!("Failed to parse private key: {}", e)))?;
        let decoding_key = DecodingKey::from_rsa_pem(decoding_key_pem)
            .map_err(|e| AppError::AuthError(format!("Failed to parse public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }

    #[allow(dead_code)]
    pub fn create_access_token(&self, user_id: &str) -> Result<String, AppError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600; // 1 hour

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration as usize,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        };

        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(|e| AppError::AuthError(format!("Failed to create access token: {}", e)))
    }

    #[allow(dead_code)]
    pub fn create_refresh_token(&self, user_id: &str) -> Result<String, AppError> {
        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 86400 * 7; // 7 days

        let claims = Claims {
            sub: user_id.to_owned(),
            exp: expiration as usize,
            iat: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as usize,
        };

        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(|e| AppError::AuthError(format!("Failed to create refresh token: {}", e)))
    }

    #[allow(dead_code)]
    pub fn validate_token(&self, token: &str) -> Result<Claims, AppError> {
        let validation = Validation::new(Algorithm::RS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::AuthError(format!("Token validation failed: {}", e)))?;

        Ok(token_data.claims)
    }
}
