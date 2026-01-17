use crate::error::AppError;
use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

#[async_trait]
pub trait ObjectStorage: Send + Sync {
    async fn put_object(&self, key: &str, data: Bytes, content_type: &str) -> Result<(), AppError>;
    async fn get_object(&self, key: &str) -> Result<Bytes, AppError>;
    async fn get_presigned_url(&self, key: &str, expires_in: Duration) -> Result<String, AppError>;
    async fn delete_object(&self, key: &str) -> Result<(), AppError>;
}
