// Integration test for login endpoint
// Run: cargo test --test test_login -- --nocapture

use serde_json::json;

#[tokio::test]
async fn test_login_endpoint() {
    // Load environment variables
    dotenvy::from_path("../.env").ok();
    
    // This test requires the database to be running
    // and migrations to be applied
    
    let response = reqwest::Client::new()
        .post("http://localhost:8080/api/v1/auth/login")
        .json(&json!({
            "username": "testuser",
            "password": "TestPass123!"
        }))
        .send()
        .await;
    
    match response {
        Ok(resp) => {
            println!("Status: {}", resp.status());
            let body = resp.text().await.unwrap();
            println!("Body: {}", body);
        }
        Err(e) => {
            eprintln!("Request failed: {}", e);
            panic!("Cannot reach server. Is it running?");
        }
    }
}
