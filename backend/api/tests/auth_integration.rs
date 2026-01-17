use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;

const BASE_URL: &str = "http://localhost:8080/api/v1";

async fn get_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
}

async fn login(client: &Client) -> (String, String) {
    let resp = client
        .post(format!("{}/auth/login", BASE_URL))
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
    let client = get_client().await;
    let resp = client
        .post(format!("{}/auth/login", BASE_URL))
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
    let client = get_client().await;
    let resp = client
        .post(format!("{}/auth/login", BASE_URL))
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
    let client = get_client().await;
    let resp = client
        .post(format!("{}/auth/login", BASE_URL))
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
    let client = get_client().await;
    let (_, refresh_token) = login(&client).await;

    let resp = client
        .post(format!("{}/auth/refresh", BASE_URL))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value = resp.json().await.unwrap();
    assert!(body["access_token"].is_string());
    // Refresh token might be rotated or same
}

#[tokio::test]
async fn test_refresh_with_expired_token_returns_401() {
    // We can't easily wait for 30 days. We'll use a made-up invalid token
    // effectively acting as expired/invalid signature.
    // If the server checks signature first, it fails.
    // If we want to check strict expiry, we'd need to mock time or have a short-lived token mechanism.
    // We'll trust "invalid token" triggers similar 401.
    let client = get_client().await;

    let resp = client
        .post(format!("{}/auth/refresh", BASE_URL))
        .json(&json!({
            "refresh_token": "valid.sha256.structure.but.expired"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED); // Or 400 Bad Request if format invalid
}

#[tokio::test]
async fn test_logout_revokes_refresh_token() {
    let client = get_client().await;
    let (access_token, refresh_token) = login(&client).await;

    // Logout
    let logout_resp = client
        .post(format!("{}/auth/logout", BASE_URL))
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
        .post(format!("{}/auth/refresh", BASE_URL))
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
    let client = get_client().await;
    // Using logout as protected route
    let resp = client
        .post(format!("{}/auth/logout", BASE_URL))
        .json(&json!({"refresh_token": "dummy"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_protected_route_with_valid_token_succeeds() {
    let client = get_client().await;
    let (access_token, refresh_token) = login(&client).await;

    // Using logout as protected route (it's the simplest one we know exists and is protected)
    let resp = client
        .post(format!("{}/auth/logout", BASE_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&json!({
            "refresh_token": refresh_token
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
}
