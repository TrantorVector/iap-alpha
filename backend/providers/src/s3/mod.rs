use domain::error::AppError;
use domain::ports::storage::ObjectStorage;
use async_trait::async_trait;
use bytes::Bytes;
use std::time::Duration;

#[derive(Clone)]
#[allow(dead_code)]
pub struct S3Storage {
    endpoint: String,
    access_key: String,
    secret_key: String,
}

impl S3Storage {
    pub fn new(endpoint: String, access_key: String, secret_key: String) -> Self {
        Self {
            endpoint,
            access_key,
            secret_key,
        }
    }
}

#[async_trait]
impl ObjectStorage for S3Storage {
    async fn put_object(
        &self,
        _key: &str,
        _data: Bytes,
        _content_type: &str,
    ) -> Result<(), AppError> {
        unimplemented!()
    }
    async fn get_object(&self, _key: &str) -> Result<Bytes, AppError> {
        unimplemented!()
    }
    async fn get_presigned_url(
        &self,
        _key: &str,
        _expires_in: Duration,
    ) -> Result<String, AppError> {
        unimplemented!()
    }
    async fn delete_object(&self, _key: &str) -> Result<(), AppError> {
        unimplemented!()
    }
}
