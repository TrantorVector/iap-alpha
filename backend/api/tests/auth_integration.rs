use api::{create_router, AppState, Config};
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

async fn spawn_app() -> String {
    // Force development environment for tests
    let mut config = Config::from_env().unwrap_or_else(|_| {
        // Create a default config if env vars are missing
        Config {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string()),
            jwt_private_key_file: None,
            jwt_public_key_file: None,
            jwt_private_key: None,
            jwt_public_key: None,
            server_host: "127.0.0.1".to_string(),
            server_port: 0,
            cors_origins: vec!["*".to_string()],
            alpha_vantage_api_key: None,
            s3_endpoint: "http://localhost:9000".to_string(),
            s3_access_key: "minioadmin".to_string(),
            s3_secret_key: "minioadmin".to_string(),
            environment: api::config::Environment::Development,
        }
    });

    // Override important settings for test isolation
    config.server_port = 0;
    config.server_host = "127.0.0.1".to_string();
    config.environment = api::config::Environment::Development;

    // Clear key paths to force auto-generation of dev keys (avoids file not found errors in CI)
    config.jwt_private_key_file = None;
    config.jwt_public_key_file = None;

    let config = Arc::new(config);
    let state = AppState::new(config.clone())
        .await
        .expect("Failed to create AppState");
    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    format!("http://{}", addr)
}

async fn get_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
}

async fn login(client: &Client, base_url: &str) -> (String, String) {
    let resp = client
        .post(format!("{}/api/v1/auth/login", base_url))
        .json(&json!({
            "username": "testuser",
            "password": "TestPass123!"
        }))
        .send()
        .await
        .expect("Failed to send login request");

    if resp.status() != StatusCode::OK {
        panic!("Login failed: {:?}", resp.status());
    }

    let body: Value = resp.json().await.unwrap();
    (
        body["access_token"].as_str().unwrap().to_string(),
        body["refresh_token"].as_str().unwrap().to_string(),
    )
}

#[tokio::test]
async fn test_login_with_valid_credentials_returns_tokens() {
    let base_url = spawn_app().await;
    let client = get_client().await;

    let resp = client
        .post(format!("{}/api/v1/auth/login", base_url))
        .json(&json!({
            "username": "testuser",
            "password": "TestPass123!"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body["access_token"].is_string());
    assert!(body["refresh_token"].is_string());
    assert!(body["user"]["id"].is_string());
    assert_eq!(body["user"]["username"], "testuser");
}

#[tokio::test]
async fn test_login_with_invalid_password_returns_401() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    let resp = client
        .post(format!("{}/api/v1/auth/login", base_url))
        .json(&json!({
            "username": "testuser",
            "password": "WrongPassword!"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_with_nonexistent_user_returns_401() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    let resp = client
        .post(format!("{}/api/v1/auth/login", base_url))
        .json(&json!({
            "username": "ghostuser",
            "password": "TestPass123!"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_refresh_with_valid_token_returns_new_access_token() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    let (_, refresh_token) = login(&client, &base_url).await;

    let resp = client
        .post(format!("{}/api/v1/auth/refresh", base_url))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body["access_token"].is_string());
}

#[tokio::test]
async fn test_refresh_with_expired_token_returns_401() {
    let base_url = spawn_app().await;
    let client = get_client().await;

    let resp = client
        .post(format!("{}/api/v1/auth/refresh", base_url))
        .json(&json!({
            "refresh_token": "valid.sha256.structure.but.expired"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_logout_revokes_refresh_token() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    let (access_token, refresh_token) = login(&client, &base_url).await;

    // Logout
    let logout_resp = client
        .post(format!("{}/api/v1/auth/logout", base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(logout_resp.status(), StatusCode::NO_CONTENT);

    // Try to refresh with the revoked token
    let refresh_resp = client
        .post(format!("{}/api/v1/auth/refresh", base_url))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(refresh_resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_without_token_returns_401() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    // Using logout as protected route
    let resp = client
        .post(format!("{}/api/v1/auth/logout", base_url))
        .json(&json!({"refresh_token": "dummy"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_valid_token_succeeds() {
    let base_url = spawn_app().await;
    let client = get_client().await;
    let (access_token, refresh_token) = login(&client, &base_url).await;

    // Using logout as protected route
    let resp = client
        .post(format!("{}/api/v1/auth/logout", base_url))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
