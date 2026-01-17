use serde::Serialize;
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Access denied: {0}")]
    ForbiddenError(String),

    #[error("Resource not found: {resource} with ID {id}")]
    NotFound { resource: &'static str, id: String },

    #[error("Validation failed: {0}")]
    ValidationError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("External provider error [{provider}]: {message}")]
    ExternalApiError { provider: String, message: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Optimistic lock conflict for {resource} {id} (current version: {current_version})")]
    OptimisticLockConflict {
        resource: &'static str,
        id: String,
        current_version: i32,
    },

    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(Serialize)]
pub struct ApiErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
}
