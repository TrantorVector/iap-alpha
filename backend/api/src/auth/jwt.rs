use domain::error::AppError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rsa::{
    pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub iss: String,
    pub scope: Vec<String>,
}

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_duration: std::time::Duration,
    refresh_token_duration: std::time::Duration,
}

impl JwtService {
    pub fn new(private_key_pem: &str, public_key_pem: &str) -> Result<Self, AppError> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
            .map_err(|e| AppError::InternalError(format!("Failed to parse private key: {}", e)))?;
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| AppError::InternalError(format!("Failed to parse public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            access_token_duration: std::time::Duration::from_secs(24 * 60 * 60), // 24 hours
            refresh_token_duration: std::time::Duration::from_secs(30 * 24 * 60 * 60), // 30 days
        })
    }

    pub fn create_access_token(
        &self,
        user_id: Uuid,
        scopes: Vec<String>,
    ) -> Result<String, AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let exp = now + self.access_token_duration.as_secs() as i64;

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp,
            iss: "iap-api".to_string(),
            scope: scopes,
        };

        encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key)
            .map_err(|e| AppError::InternalError(format!("Failed to create access token: {}", e)))
    }

    pub fn create_refresh_token(&self, user_id: Uuid) -> Result<(String, String), AppError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let exp = now + self.refresh_token_duration.as_secs() as i64;

        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp,
            iss: "iap-api".to_string(),
            scope: vec!["refresh".to_string()],
        };

        let token =
            encode(&Header::new(Algorithm::RS256), &claims, &self.encoding_key).map_err(|e| {
                AppError::InternalError(format!("Failed to create refresh token: {}", e))
            })?;

        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let hash = hex::encode(hasher.finalize());

        Ok((token, hash))
    }

    pub fn validate_access_token(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["iap-api"]);

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::AuthError(format!("Invalid token: {}", e)))?;

        Ok(token_data.claims)
    }

    pub fn decode_without_validating(&self, token: &str) -> Result<Claims, AppError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&["iap-api"]);
        validation.validate_exp = false;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| AppError::AuthError(format!("Failed to decode token: {}", e)))?;

        Ok(token_data.claims)
    }

    /// Hash a token using SHA256 (for storing/looking up refresh tokens)
    pub fn hash_token(&self, token: &str) -> Result<String, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        Ok(hex::encode(hasher.finalize()))
    }

    pub fn generate_dev_keypair() -> (String, String) {
        let mut rng = rand::thread_rng();
        let bits = 2048;
        let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
        let pub_key = RsaPublicKey::from(&priv_key);

        let priv_pem = priv_key
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("failed to encode private key");
        let pub_pem = pub_key
            .to_pkcs1_pem(rsa::pkcs8::LineEnding::LF)
            .expect("failed to encode public key");

        (priv_pem.to_string(), pub_pem.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_flow() {
        let (priv_key, pub_key) = JwtService::generate_dev_keypair();
        let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

        let user_id = Uuid::new_v4();
        let token = service
            .create_access_token(user_id, vec!["read".to_string()])
            .expect("Failed to create token");

        let claims = service
            .validate_access_token(&token)
            .expect("Failed to validate token");
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.scope, vec!["read".to_string()]);
        assert_eq!(claims.iss, "iap-api");
    }

    #[test]
    fn test_refresh_token() {
        let (priv_key, pub_key) = JwtService::generate_dev_keypair();
        let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

        let user_id = Uuid::new_v4();
        let (token, hash) = service
            .create_refresh_token(user_id)
            .expect("Failed to create refresh token");

        assert!(!token.is_empty());
        assert!(!hash.is_empty());

        // Validate the refresh token as a normal token (since it's a valid signed JWT)
        let claims = service
            .validate_access_token(&token)
            .expect("Failed to validate refresh token");
        assert_eq!(claims.sub, user_id.to_string());
    }
}
