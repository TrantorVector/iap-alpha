use crate::common::{setup_test_db, TestApp};
use reqwest::StatusCode;
use serde_json::{json, Value};
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_screener_crud_flow() {
    let app = TestApp::spawn().await;
    setup_test_db(&app.db_pool).await;

    let client = reqwest::Client::new();
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(user_id)
    .bind(format!("scr_{}", user_id))
    .bind(format!("scr_{}@example.com", user_id))
    .bind("hash")
    .execute(&app.db_pool)
    .await
    .unwrap();
    let token = app.generate_token(user_id);

    // 1. Create a screener
    let create_resp = client
        .post(format!("{}/api/v1/screeners", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": "High Growth Screener",
            "description": "Companies with >15% growth",
            "filter_criteria": {
                "market_cap_min": 1000000000,
                "industries": ["Technology"]
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_resp.status(), StatusCode::CREATED);
    let screener: Value = create_resp.json().await.unwrap();
    let screener_id = screener["id"].as_str().unwrap();

    // 2. List screeners
    let list_resp = client
        .get(format!("{}/api/v1/screeners", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(list_resp.status(), StatusCode::OK);
    let screeners: Vec<Value> = list_resp.json().await.unwrap();
    assert!(screeners
        .iter()
        .any(|s| s["id"].as_str().unwrap() == screener_id));

    // 3. Update screener
    let update_resp = client
        .put(format!("{}/api/v1/screeners/{}", app.address, screener_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": "Updated Growth Screener",
            "description": "More specific",
            "filter_criteria": {
                "market_cap_min": 2000000000,
                "industries": ["Technology", "Healthcare"]
            }
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(update_resp.status(), StatusCode::OK);

    // 4. Delete screener
    let delete_resp = client
        .delete(format!("{}/api/v1/screeners/{}", app.address, screener_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(delete_resp.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn test_run_screener_returns_results() {
    let app = TestApp::spawn().await;
    setup_test_db(&app.db_pool).await;

    // Seed some data
    let apple_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO companies (id, symbol, name, exchange, industry) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(apple_id)
    .bind("AAPL")
    .bind("Apple Inc")
    .bind("NASDAQ")
    .bind("Technology")
    .execute(&app.db_pool)
    .await
    .unwrap();

    let client = reqwest::Client::new();
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW())",
    )
    .bind(user_id)
    .bind(format!("scr_run_{}", user_id))
    .bind(format!("scr_run_{}@example.com", user_id))
    .bind("hash")
    .execute(&app.db_pool)
    .await
    .unwrap();
    let token = app.generate_token(user_id);

    // Create a screener
    let create_resp = client
        .post(format!("{}/api/v1/screeners", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "title": "Tech Screener",
            "filter_criteria": {
                "industries": ["Technology"]
            }
        }))
        .send()
        .await
        .unwrap();

    let screener: Value = create_resp.json().await.unwrap();
    let screener_id = screener["id"].as_str().unwrap();

    // Run the screener
    let run_resp = client
        .post(format!(
            "{}/api/v1/screeners/{}/run",
            app.address, screener_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(run_resp.status(), StatusCode::OK);
    let body: Value = run_resp.json().await.unwrap();
    let results = body["results"].as_array().unwrap();
    assert!(results.iter().any(|r| r["symbol"] == "AAPL"));
}
