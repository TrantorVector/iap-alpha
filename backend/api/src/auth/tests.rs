use super::jwt::{Claims, JwtService};
use super::password::{hash_password, verify_password};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

#[test]
fn test_hash_password_creates_valid_hash() {
    let password = "secure_password_123";
    let hash = hash_password(password).expect("Failed to hash password");
    assert!(!hash.is_empty());
    assert!(hash.starts_with("$argon2id"));
}

#[test]
fn test_verify_correct_password_succeeds() {
    let password = "secure_password_123";
    let hash = hash_password(password).expect("Failed to hash password");

    let is_valid = verify_password(password, &hash).expect("Failed to verify password");
    assert!(is_valid);
}

#[test]
fn test_verify_wrong_password_fails() {
    let password = "secure_password_123";
    let hash = hash_password(password).expect("Failed to hash password");

    let is_valid = verify_password("wrong_password", &hash).expect("Failed to verify password");
    assert!(!is_valid);
}

#[test]
fn test_hash_is_not_deterministic() {
    let password = "same_password";
    let hash1 = hash_password(password).expect("Failed to hash password 1");
    let hash2 = hash_password(password).expect("Failed to hash password 2");

    // Argon2 uses a random salt by default, so hashes should be different
    assert_ne!(hash1, hash2);
}

#[test]
fn test_create_access_token_is_valid() {
    let (priv_key, pub_key) = JwtService::generate_dev_keypair();
    let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

    let user_id = Uuid::new_v4();
    let token = service
        .create_access_token(user_id, vec!["read".to_string()])
        .expect("Failed to create token");

    let claims = service
        .validate_access_token(&token)
        .expect("Valid token rejected");
    assert_eq!(claims.sub, user_id.to_string());
}

#[test]
fn test_access_token_contains_correct_claims() {
    let (priv_key, pub_key) = JwtService::generate_dev_keypair();
    let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

    let user_id = Uuid::new_v4();
    let scope = vec!["read".to_string(), "write".to_string()];
    let token = service
        .create_access_token(user_id, scope.clone())
        .expect("Failed to create token");

    let claims = service
        .validate_access_token(&token)
        .expect("Token validation failed");

    assert_eq!(claims.sub, user_id.to_string());
    assert_eq!(claims.iss, "iap-api");
    assert_eq!(claims.scope, scope);
    // basic check that exp is in the future
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    assert!(claims.exp > now);
}

#[test]
fn test_expired_token_is_rejected() {
    let (priv_key, pub_key) = JwtService::generate_dev_keypair();
    let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

    // Manually create an expired token
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        iat: now - 7200, // 2 hours ago
        exp: now - 3600, // expired 1 hour ago
        iss: "iap-api".to_string(),
        scope: vec![],
    };

    let encoding_key = EncodingKey::from_rsa_pem(priv_key.as_bytes()).unwrap();
    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).unwrap();

    let result = service.validate_access_token(&token);
    assert!(result.is_err());
    // Ideally check error type, but checking is_err is sufficient for now
}

#[test]
fn test_invalid_signature_is_rejected() {
    let (priv_key1, _) = JwtService::generate_dev_keypair();
    let (_, pub_key2) = JwtService::generate_dev_keypair();

    // Sign with key1, verify with key2
    let service_verifier =
        JwtService::new(&priv_key1, &pub_key2).expect("Failed to create service");
    // Actually we need to sign with key1. But JwtService takes priv/pub pair.
    // If we pass priv_key1 and pub_key2, creating token uses priv_key1.
    // Validating uses pub_key2. They don't match.

    let user_id = Uuid::new_v4();
    let token = service_verifier
        .create_access_token(user_id, vec![])
        .expect("Failed to create token");

    let result = service_verifier.validate_access_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_wrong_algorithm_is_rejected() {
    let (priv_key, pub_key) = JwtService::generate_dev_keypair();
    let service = JwtService::new(&priv_key, &pub_key).expect("Failed to create service");

    // Create token with HS256
    let claims = Claims {
        sub: Uuid::new_v4().to_string(),
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64,
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600,
        iss: "iap-api".to_string(),
        scope: vec![],
    };

    // Use a random secret for HS256
    let key = EncodingKey::from_secret(b"secret");
    let token = encode(&Header::new(Algorithm::HS256), &claims, &key).unwrap();

    let result = service.validate_access_token(&token);
    assert!(result.is_err());
}

#[test]
fn test_token_signed_with_wrong_key_is_rejected() {
    // This is similar to invalid signature
    let (priv_key1, pub_key1) = JwtService::generate_dev_keypair();
    let (priv_key2, pub_key2) = JwtService::generate_dev_keypair();

    let service1 = JwtService::new(&priv_key1, &pub_key1).unwrap();
    let service2 = JwtService::new(&priv_key2, &pub_key2).unwrap();

    let token = service1
        .create_access_token(Uuid::new_v4(), vec![])
        .unwrap();

    // Validate with service2 (different key)
    let result = service2.validate_access_token(&token);
    assert!(result.is_err());
}
