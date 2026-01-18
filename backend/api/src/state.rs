use crate::auth::jwt::JwtService;
use crate::config::{Config, Environment};
use db::PgPool;
use domain::error::AppError;
use domain::ports::market_data::MarketDataProvider;
use domain::ports::storage::ObjectStorage;
use providers::alpha_vantage::AlphaVantageClient;
use providers::mock::{MockMarketDataProvider, MockObjectStorage};
use providers::s3::S3Storage;
use std::fs;
use std::sync::Arc;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db: PgPool,
    pub config: Arc<Config>,
    pub market_data: Arc<dyn MarketDataProvider>,
    pub storage: Arc<dyn ObjectStorage>,
    pub jwt_service: Arc<JwtService>,
}

impl AppState {
    pub async fn new(config: Arc<Config>) -> Result<Self, AppError> {
        // Init DB
        let db = db::init_pool(&config.database_url)
            .await
            .map_err(|e| AppError::InternalError(format!("Database init failed: {}", e)))?;

        // Init Providers
        let market_data: Arc<dyn MarketDataProvider> = match config.environment {
            Environment::Production | Environment::Staging => {
                let api_key = config.alpha_vantage_api_key.clone().ok_or_else(|| {
                    AppError::InternalError("Alpha Vantage API key missing".into())
                })?;
                Arc::new(AlphaVantageClient::new(api_key))
            }
            Environment::Development => Arc::new(MockMarketDataProvider::new()),
        };

        let storage: Arc<dyn ObjectStorage> = match config.environment {
            Environment::Production | Environment::Staging => Arc::new(S3Storage::new(
                config.s3_endpoint.clone(),
                config.s3_access_key.clone(),
                config.s3_secret_key.clone(),
            ).await),
            Environment::Development => {
                // For development, we can also use S3Storage if configured, or Mock.
                // The config logic defaults S3 endpoint for development, so maybe we SHOULD use S3Storage even in dev?
                // But the valid 'mock' usage suggests using Mock.
                // However, MinIO is part of the setup (docker-compose).
                // If I use MockObjectStorage, I won't use MinIO.
                // But prompt said "Create appropriate provider implementations".
                // If I have a real S3 endpoint in config, I should use it?
                // But let's stick to Mock for explicit Development env to be safe, or maybe check if endpoint is localhost?
                // Let's use S3Storage if we are in Dev but S3 is configured?
                // Config ALWAYS has s3_endpoint (defaults to localhost:9000 in dev).
                // So I should probably use S3Storage in Development too if I want to use MinIO.
                // But keeping it simple with MockObjectStorage for now as per "Mock" implication.
                // Actually, if I use Mock, I don't test MinIO.
                // But I'll stick to the code I drafted: use MockObjectStorage for Development to satisfy likely intent of "Mock" providers.
                // One correction: If I want to verify "Step 5.5", maybe I should just use S3Storage for all?
                // But I implemented MockObjectStorage.
                // Let's use Mock for Development.
                Arc::new(MockObjectStorage::new())
            }
        };

        // Init JWT
        let (jwt_private_key_str, jwt_public_key_str) = if let Some(ref path) =
            config.jwt_private_key_file
        {
            let priv_key = fs::read_to_string(path).map_err(|e| {
                AppError::InternalError(format!(
                    "Failed to read JWT private key file from {}: {}",
                    path, e
                ))
            })?;

            let pub_path = config.jwt_public_key_file.as_ref().ok_or_else(|| {
                AppError::InternalError(
                    "JWT public key file path missing while private key path is set".into(),
                )
            })?;

            let pub_key = fs::read_to_string(pub_path).map_err(|e| {
                AppError::InternalError(format!(
                    "Failed to read JWT public key file from {}: {}",
                    pub_path, e
                ))
            })?;

            (priv_key, pub_key)
        } else if let Some(ref priv_inline) = config.jwt_private_key {
            let pub_inline = config.jwt_public_key.as_ref().ok_or_else(|| {
                AppError::InternalError("JWT public key missing while private key is set".into())
            })?;

            (priv_inline.clone(), pub_inline.clone())
        } else if config.environment == Environment::Development {
            tracing::warn!("JWT keys not configured. Generating temporary keys for development...");
            let (priv_key, pub_key) = JwtService::generate_dev_keypair();
            (priv_key, pub_key)
        } else {
            return Err(AppError::InternalError(
                "JWT keys not configured and not in development mode".into(),
            ));
        };

        let jwt_service = Arc::new(JwtService::new(&jwt_private_key_str, &jwt_public_key_str)?);

        Ok(Self {
            db,
            config,
            market_data,
            storage,
            jwt_service,
        })
    }
}
