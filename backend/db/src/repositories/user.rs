/// User repository for authentication and user management
use crate::models::{RefreshToken, User, UserPreferences};
use crate::{DbError, DbResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

/// Request to create a new user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub timezone: Option<String>,
}

/// Request to update user preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferencesUpdate {
    pub document_row_order: Option<serde_json::Value>,
    pub default_period_count: Option<i32>,
    pub default_period_type: Option<String>,
    pub theme: Option<String>,
}

/// User repository
pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    /// Create a new user repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // User CRUD Operations
    // =========================================================================

    /// Find user by username
    pub async fn find_by_username(&self, username: &str) -> DbResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, display_name, timezone,
                   is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(user)
    }

    /// Find user by email
    pub async fn find_by_email(&self, email: &str) -> DbResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, display_name, timezone,
                   is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(user)
    }

    /// Find user by ID
    pub async fn find_by_id(&self, id: Uuid) -> DbResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, username, email, password_hash, display_name, timezone,
                   is_active, created_at, updated_at, last_login_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(&self, req: CreateUserRequest) -> DbResult<User> {
        let id = Uuid::new_v4();
        let now = Utc::now();
        let timezone = req.timezone.unwrap_or_else(|| "Asia/Kolkata".to_string());

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (
                id, username, email, password_hash, display_name,
                timezone, is_active, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, username, email, password_hash, display_name, timezone,
                      is_active, created_at, updated_at, last_login_at
            "#,
        )
        .bind(id)
        .bind(&req.username)
        .bind(&req.email)
        .bind(&req.password_hash)
        .bind(&req.display_name)
        .bind(&timezone)
        .bind(true) // is_active
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(user)
    }

    /// Update user password
    pub async fn update_password(&self, id: Uuid, password_hash: &str) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(password_hash)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    /// Update last login timestamp
    pub async fn update_last_login(&self, id: Uuid) -> DbResult<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET last_login_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(())
    }

    // =========================================================================
    // Refresh Token Operations
    // =========================================================================

    /// Create a new refresh token
    pub async fn create_refresh_token(
        &self,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> DbResult<Uuid> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at, revoked)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .bind(false)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(id)
    }

    /// Find a valid (non-revoked, non-expired) refresh token
    pub async fn find_valid_refresh_token(
        &self,
        token_hash: &str,
    ) -> DbResult<Option<RefreshToken>> {
        let token = sqlx::query_as::<_, RefreshToken>(
            r#"
            SELECT id, user_id, token_hash, expires_at, revoked,
                   device_info, ip_address, created_at
            FROM refresh_tokens
            WHERE token_hash = $1
              AND revoked = false
              AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(token)
    }

    /// Revoke a specific refresh token
    pub async fn revoke_refresh_token(&self, id: Uuid) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = true
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound("Refresh token not found".to_string()));
        }

        Ok(())
    }

    /// Revoke all refresh tokens for a user
    pub async fn revoke_all_user_tokens(&self, user_id: Uuid) -> DbResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE refresh_tokens
            SET revoked = true
            WHERE user_id = $1 AND revoked = false
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected())
    }

    /// Clean up expired tokens (should be called periodically)
    pub async fn clean_expired_tokens(&self) -> DbResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM refresh_tokens
            WHERE expires_at < NOW() - INTERVAL '7 days'
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected())
    }

    // =========================================================================
    // User Preferences Operations
    // =========================================================================

    /// Get user preferences
    pub async fn get_preferences(&self, user_id: Uuid) -> DbResult<Option<UserPreferences>> {
        let prefs = sqlx::query_as::<_, UserPreferences>(
            r#"
            SELECT id, user_id, document_row_order, default_period_count,
                   default_period_type, theme, updated_at
            FROM user_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(prefs)
    }

    /// Upsert (insert or update) user preferences
    pub async fn upsert_preferences(
        &self,
        user_id: Uuid,
        update: UserPreferencesUpdate,
    ) -> DbResult<UserPreferences> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Set defaults for optional fields
        let document_row_order = update.document_row_order.unwrap_or_else(|| {
            serde_json::json!(["investor_presentation", "earnings_call_transcript", "earnings_release"])
        });
        let default_period_count = update.default_period_count.unwrap_or(4);
        let default_period_type = update.default_period_type.unwrap_or_else(|| "quarterly".to_string());
        let theme = update.theme.unwrap_or_else(|| "light".to_string());

        let prefs = sqlx::query_as::<_, UserPreferences>(
            r#"
            INSERT INTO user_preferences (
                id, user_id, document_row_order, default_period_count,
                default_period_type, theme, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id) DO UPDATE SET
                document_row_order = COALESCE($3, user_preferences.document_row_order),
                default_period_count = COALESCE($4, user_preferences.default_period_count),
                default_period_type = COALESCE($5, user_preferences.default_period_type),
                theme = COALESCE($6, user_preferences.theme),
                updated_at = $7
            RETURNING id, user_id, document_row_order, default_period_count,
                      default_period_type, theme, updated_at
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(document_row_order)
        .bind(default_period_count)
        .bind(default_period_type)
        .bind(theme)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(prefs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL instance
    async fn test_create_and_find_user() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());
        
        let pool = crate::init_pool(&database_url).await.unwrap();
        let repo = UserRepository::new(pool);

        let req = CreateUserRequest {
            username: "testuser123".to_string(),
            email: "test123@example.com".to_string(),
            password_hash: "hashedpassword".to_string(),
            display_name: Some("Test User".to_string()),
            timezone: None,
        };

        let user = repo.create(req).await.unwrap();
        assert_eq!(user.username, "testuser123");

        let found = repo.find_by_username("testuser123").await.unwrap();
        assert!(found.is_some());
    }
}
