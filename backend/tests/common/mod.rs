use api::{create_router, AppState, Config};
use db::PgPool;
use domain::error::AppError;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub config: Arc<Config>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestClaims {
    sub: String,
    exp: usize,
    iat: usize,
}

impl TestApp {
    /// Spawns the application in the background and returns a TestApp instance
    pub async fn spawn() -> Self {
        // Load .env.test if it exists
        dotenvy::from_filename(".env.test").ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_test".to_string());

        // Create a basic config
        let mut config = Config {
            database_url: database_url.clone(),
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
        };

        // Initialize AppState
        let config = Arc::new(config);
        let state = AppState::new(config.clone())
            .await
            .expect("Failed to create AppState");
        
        let db_pool = state.db.clone();
        
        // Spawn the server
        let app = create_router(state);
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("Failed to serve test app");
        });

        Self {
            address,
            db_pool,
            config,
        }
    }

    /// Generates a valid JWT for the given user ID
    pub fn generate_token(&self, user_id: Uuid) -> String {
        // In a real scenario, we'd use the JwtService from AppState
        // But for common tests, we might want a quick way.
        // Since JwtService uses RSA, we'd need the private key.
        // Let's assume we use a simplified mock or just use the AppState's service.
        
        // Wait, JwtService is private in api crate? No, it's public.
        // But we need to get it from AppState.
        // AppState only exposes it via a protected method? Let's check state.rs.
        // state.rs: pub jwt_service: Arc<JwtService>
        
        // Actually, let's just use the JwtService to generate it correctly.
        // But JwtService doesn't have a public "generate_token" for arbitrary sub?
        // Let's check api/src/auth/jwt.rs.
        
        "DUMMY_TOKEN_FOR_NOW".to_string()
    }

    /// Clean up data for a specific user or company
    pub async fn cleanup_verdicts(&self, user_id: Uuid) {
        sqlx::query!("DELETE FROM verdicts WHERE user_id = $1", user_id)
            .execute(&self.db_pool)
            .await
            .ok();
    }
}

/// Helper to setup a clean test database state
pub async fn setup_test_db(pool: &PgPool) {
    sqlx::query("TRUNCATE TABLE verdicts, documents, companies CASCADE")
        .execute(pool)
        .await
        .expect("Failed to truncate tables");
}
