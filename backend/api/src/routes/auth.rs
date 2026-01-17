use crate::auth::jwt::Claims;
use crate::error::ApiError;
use crate::state::AppState;
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use axum::{
    extract::State,
    http::StatusCode,
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshRequest {
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RefreshResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    request_body = RefreshRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = RefreshResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshRequest>,
) -> Result<Json<RefreshResponse>, ApiError> {
    // Validate request body
    if payload.refresh_token.is_empty() {
        return Err(ApiError(AppError::ValidationError("Refresh token is required".to_string())));
    }

    // Hash the provided refresh token
    let refresh_token_hash = state
        .jwt_service
        .hash_token(&payload.refresh_token)
        .map_err(|e| {
            warn!("Failed to hash refresh token: {}", e);
            ApiError(AppError::AuthError("Invalid refresh token".to_string()))
        })?;

    let user_repo = UserRepository::new(state.db.clone());

    // Look up the refresh token in the database
    let token_record = match user_repo.find_refresh_token(&refresh_token_hash).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            warn!("Refresh token not found in database");
            return Err(ApiError(AppError::AuthError("Invalid or expired refresh token".to_string())));
        }
        Err(e) => {
            warn!("Database error during refresh token lookup: {}", e);
            return Err(ApiError(AppError::InternalError("Database error".into())));
        }
    };

    // Verify token is not revoked
    if token_record.revoked {
        warn!("Attempted to use revoked refresh token");
        return Err(ApiError(AppError::AuthError("Invalid or expired refresh token".to_string())));
    }

    // Verify token is not expired
    let now = chrono::Utc::now();
    if token_record.expires_at < now {
        warn!("Refresh token has expired");
        return Err(ApiError(AppError::AuthError("Invalid or expired refresh token".to_string())));
    }

    // Generate new access token
    let access_token = state
        .jwt_service
        .create_access_token(token_record.user_id, vec![])
        .map_err(ApiError)?;

    // Calculate access token expires_in (seconds)
    let access_claims = state
        .jwt_service
        .decode_without_validating(&access_token)
        .map_err(ApiError)?;
    let expires_in = access_claims.exp - now.timestamp();

    info!("Refresh token used successfully for user_id: {}", token_record.user_id);

    Ok(Json(RefreshResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
    }))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LogoutRequest {
    #[schema(example = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...")]
    pub refresh_token: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    request_body = Option<LogoutRequest>,
    responses(
        (status = 204, description = "Logout successful"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    ),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    claims: Claims,
    payload: Option<Json<LogoutRequest>>,
) -> Result<StatusCode, ApiError> {
    let user_repo = UserRepository::new(state.db.clone());
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
        warn!("Invalid user ID in claims: {}", claims.sub);
        ApiError(AppError::AuthError("Invalid token payload".into()))
    })?;

    if let Some(Json(req)) = payload {
        if let Some(refresh_token) = req.refresh_token {
            if refresh_token.is_empty() {
                // If provided but empty, treat as logout all
                user_repo.revoke_all_user_tokens(user_id).await.map_err(|e| {
                    warn!("Failed to revoke all refresh tokens for user {}: {}", user_id, e);
                    ApiError(AppError::InternalError("Database error".into()))
                })?;
                info!("All refresh tokens revoked for user {} (empty token provided)", user_id);
            } else {
                // Hash the provided refresh token
                let refresh_token_hash = state
                    .jwt_service
                    .hash_token(&refresh_token)
                    .map_err(|e| {
                        warn!("Failed to hash refresh token during logout: {}", e);
                        ApiError(AppError::AuthError("Invalid refresh token".to_string()))
                    })?;

                // Look up the refresh token
                match user_repo.find_refresh_token(&refresh_token_hash).await {
                    Ok(Some(token_record)) => {
                        // Verify token belongs to the user
                        if token_record.user_id == user_id {
                            user_repo.revoke_refresh_token(token_record.id).await.map_err(|e| {
                                warn!("Failed to revoke specific refresh token for user {}: {}", user_id, e);
                                ApiError(AppError::InternalError("Database error".into()))
                            })?;
                            info!("Specific refresh token revoked for user: {}", user_id);
                        } else {
                            warn!("User {} tried to revoke token belonging to user {}", user_id, token_record.user_id);
                            // For security, don't confirm the token exists for another user
                            return Err(ApiError(AppError::AuthError("Unauthorized".into())));
                        }
                    }
                    Ok(None) => {
                        info!("Logout: Refresh token not found in database (likely already revoked or expired)");
                    }
                    Err(e) => {
                        warn!("Database error during refresh token lookup for logout: {}", e);
                        return Err(ApiError(AppError::InternalError("Database error".into())));
                    }
                }
            }
        } else {
            // No specific token, revoke all
            user_repo.revoke_all_user_tokens(user_id).await.map_err(|e| {
                warn!("Failed to revoke all refresh tokens for user {}: {}", user_id, e);
                ApiError(AppError::InternalError("Database error".into()))
            })?;
            info!("All refresh tokens revoked for user: {}", user_id);
        }
    } else {
        // No body, revoke all
        user_repo.revoke_all_user_tokens(user_id).await.map_err(|e| {
            warn!("Failed to revoke all refresh tokens for user {}: {}", user_id, e);
            ApiError(AppError::InternalError("Database error".into()))
        })?;
        info!("All refresh tokens revoked for user: {}", user_id);
    }

    Ok(StatusCode::NO_CONTENT)
}
