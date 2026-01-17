use crate::error::ApiError;
use crate::state::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use db::repositories::user::UserRepository;
use domain::error::AppError;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "admin")]
    pub username: String,
    #[schema(example = "password123")]
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    // Validate request body
    if payload.username.is_empty() {
        return Err(ApiError(AppError::ValidationError("Username is required".to_string())));
    }
    if payload.password.is_empty() {
        return Err(ApiError(AppError::ValidationError("Password is required".to_string())));
    }

    let user_repo = UserRepository::new(state.db.clone());

    // Find user by username
    let user = match user_repo.find_by_username(&payload.username).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            warn!("Login failed: User '{}' not found", payload.username);
            // Return generic error to avoid user enumeration
            return Err(ApiError(AppError::AuthError("Invalid username or password".to_string())));
        }
        Err(e) => {
            warn!("Database error during login for '{}': {}", payload.username, e);
            return Err(ApiError(AppError::InternalError("Database error".into())));
        }
    };

    // Verify password hash
    let parsed_hash = PasswordHash::new(&user.password_hash).map_err(|e| {
        warn!("Invalid password hash stored for user '{}': {}", payload.username, e);
        ApiError(AppError::InternalError("Stored password hash is invalid".to_string()))
    })?;

    if let Err(_) = Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash) {
        warn!("Login failed: Invalid password for user '{}'", payload.username);
        return Err(ApiError(AppError::AuthError("Invalid username or password".to_string())));
    }

    // Generate tokens
    let access_token = state.jwt_service.create_access_token(user.id, vec![]).map_err(ApiError)?;
    let (refresh_token, refresh_token_hash) = state.jwt_service.create_refresh_token(user.id).map_err(ApiError)?;

    // Calculate expiration for refresh token (needed for DB)
    // We decode the token to get the expiration time
    let refresh_claims = state.jwt_service.decode_without_validating(&refresh_token).map_err(ApiError)?;
    let expires_at = chrono::DateTime::<chrono::Utc>::from_timestamp(refresh_claims.exp, 0)
        .ok_or_else(|| ApiError(AppError::InternalError("Invalid token expiration".to_string())))?;

    // Store refresh token hash in database
    user_repo
        .create_refresh_token(user.id, &refresh_token_hash, expires_at)
        .await
        .map_err(|e| {
            warn!("Failed to store refresh token for user '{}': {}", payload.username, e);
            ApiError(AppError::InternalError("Failed to create refresh session".to_string()))
        })?;

    // Update last login
    if let Err(e) = user_repo.update_last_login(user.id).await {
        warn!("Failed to update last login for user '{}': {}", payload.username, e);
        // Not critical, continue
    }

    info!("User '{}' logged in successfully", payload.username);

    // Calculate access token expires_in (seconds)
    let access_claims = state.jwt_service.decode_without_validating(&access_token).map_err(ApiError)?;
    let now = chrono::Utc::now().timestamp();
    let expires_in = access_claims.exp - now;

    Ok(Json(LoginResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
        },
    }))
}

pub async fn refresh_token(State(_state): State<AppState>) -> Result<impl IntoResponse, ApiError> {
    Ok(Json("Refresh token placeholder"))
}
