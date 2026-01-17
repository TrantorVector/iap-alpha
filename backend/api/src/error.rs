use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use core::error::{ApiErrorResponse, AppError, ErrorDetails};
use serde_json::json;
use tracing::{error, warn};

#[allow(dead_code)]
pub struct ApiError(pub AppError);

impl From<AppError> for ApiError {
    fn from(inner: AppError) -> Self {
        ApiError(inner)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, details) = match &self.0 {
            AppError::AuthError(_) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", None),
            AppError::ForbiddenError(_) => (StatusCode::FORBIDDEN, "FORBIDDEN", None),
            AppError::NotFound { .. } => (StatusCode::NOT_FOUND, "NOT_FOUND", None),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", None),
            AppError::DatabaseError(e) => {
                error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", None)
            }
            AppError::ExternalApiError { provider, message } => {
                warn!("External API error [{}]: {}", provider, message);
                (
                    StatusCode::BAD_GATEWAY,
                    "EXTERNAL_API_ERROR",
                    Some(json!({ "provider": provider })),
                )
            }
            AppError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", None)
            }
            AppError::OptimisticLockConflict { .. } => (StatusCode::CONFLICT, "CONFLICT", None),
            AppError::InternalError(e) => {
                error!("Internal error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", None)
            }
        };

        // Log client errors as warnings, server errors as errors
        if status.is_client_error() {
            warn!(%status, error = %self.0, "Client error");
        } else if status.is_server_error() {
            error!(%status, error = %self.0, "Server error");
        }

        let body = Json(ApiErrorResponse {
            error: ErrorDetails {
                code: code.to_string(),
                message: self.0.to_string(),
                details,
            },
        });

        (status, body).into_response()
    }
}
