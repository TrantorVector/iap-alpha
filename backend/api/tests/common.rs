use api::auth::jwt::JwtService;
use api::{create_router, AppState, Config};
use db::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub config: Arc<Config>,
    pub jwt_service: Arc<JwtService>,
}

impl TestApp {
    /// Spawns the application in the background and returns a TestApp instance
    pub async fn spawn() -> Self {
        // Load .env.test if it exists
        dotenvy::from_filename(".env.test").ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_test".to_string());

        // Create a basic config
        let config = Config {
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
        let jwt_service = state.jwt_service.clone();

        // Spawn the server
        let app = create_router(state);
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("Failed to bind random port");
        let port = listener.local_addr().unwrap().port();
        let address = format!("http://127.0.0.1:{}", port);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("Failed to serve test app");
        });

        Self {
            address,
            db_pool,
            config,
            jwt_service,
        }
    }

    /// Generates a valid JWT for the given user ID
    pub fn generate_token(&self, user_id: Uuid) -> String {
        self.jwt_service
            .create_access_token(user_id, vec!["user".to_string()])
            .expect("Failed to create access token")
    }

    /// Clean up data for a specific user or company
    pub async fn cleanup_verdicts(&self, user_id: Uuid) {
        sqlx::query("DELETE FROM verdicts WHERE user_id = $1")
            .bind(user_id)
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
