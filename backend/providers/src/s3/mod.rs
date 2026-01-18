use async_trait::async_trait;
use bytes::Bytes;
use domain::error::AppError;
use domain::ports::storage::ObjectStorage;
use std::time::Duration;

use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct S3Storage {
    client: Client,
    bucket: String,
}

impl S3Storage {
    pub async fn new(endpoint: String, access_key: String, secret_key: String) -> Self {
        let credentials = Credentials::new(access_key, secret_key, None, None, "static");
        let region = Region::new("us-east-1"); // Standard default for MinIO/S3

        let config = aws_config::defaults(aws_config::BehaviorVersion::latest())
            .credentials_provider(credentials)
            .region(region)
            .endpoint_url(endpoint)
            .load()
            .await;

        let client = Client::new(&config);

        // In a real app, this would come from config.
        // For development/alpha we'll use a standard bucket name.
        let bucket = std::env::var("S3_BUCKET").unwrap_or_else(|_| "iap-documents".to_string());

        Self { client, bucket }
    }
}

#[async_trait]
impl ObjectStorage for S3Storage {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> Result<(), AppError> {
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(data.into())
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("S3 put error: {}", e)))?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Bytes, AppError> {
        let output = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("S3 get error: {}", e)))?;

        let data = output
            .body
            .collect()
            .await
            .map_err(|e| AppError::InternalError(format!("S3 body collect error: {}", e)))?
            .into_bytes();

        Ok(data)
    }

    async fn get_presigned_url(&self, key: &str, expires_in: Duration) -> Result<String, AppError> {
        let presigning_config = PresigningConfig::builder()
            .expires_in(expires_in)
            .build()
            .map_err(|e| AppError::InternalError(format!("Presigning config error: {}", e)))?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| AppError::InternalError(format!("Presigning error: {}", e)))?;

        Ok(presigned_request.uri().to_string())
    }

    async fn delete_object(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| AppError::InternalError(format!("S3 delete error: {}", e)))?;
        Ok(())
    }
}
