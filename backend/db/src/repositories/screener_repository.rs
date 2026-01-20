use crate::error::{DbError, DbResult};
use crate::models::screener::Screener;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScreener {
    pub title: String,
    pub description: Option<String>,
    pub filter_criteria: serde_json::Value,
    pub sort_config: Option<serde_json::Value>,
    pub display_columns: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScreener {
    pub title: Option<String>,
    pub description: Option<String>,
    pub filter_criteria: Option<serde_json::Value>,
    pub sort_config: Option<serde_json::Value>,
    pub display_columns: Option<serde_json::Value>,
}

#[derive(Clone)]
pub struct ScreenerRepository {
    pool: PgPool,
}

impl ScreenerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> DbResult<Vec<Screener>> {
        let screeners = sqlx::query_as::<_, Screener>(
            r#"
            SELECT * FROM screeners
            WHERE user_id = $1
            ORDER BY display_order ASC, created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(screeners)
    }

    pub async fn find_by_id(&self, id: Uuid, user_id: Uuid) -> DbResult<Option<Screener>> {
        let screener = sqlx::query_as::<_, Screener>(
            r#"
            SELECT * FROM screeners
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(screener)
    }

    pub async fn create(&self, user_id: Uuid, screener: CreateScreener) -> DbResult<Screener> {
        let created_screener = sqlx::query_as::<_, Screener>(
            r#"
            INSERT INTO screeners (
                user_id, title, description, filter_criteria, sort_config, display_columns
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
        )
        .bind(user_id)
        .bind(screener.title)
        .bind(screener.description)
        .bind(screener.filter_criteria)
        .bind(screener.sort_config)
        .bind(screener.display_columns)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(created_screener)
    }

    pub async fn update(
        &self,
        id: Uuid,
        user_id: Uuid,
        screener: UpdateScreener,
    ) -> DbResult<Screener> {
        let updated_screener = sqlx::query_as::<_, Screener>(
            r#"
            UPDATE screeners
            SET
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                filter_criteria = COALESCE($5, filter_criteria),
                sort_config = COALESCE($6, sort_config),
                display_columns = COALESCE($7, display_columns),
                updated_at = NOW()
            WHERE id = $1 AND user_id = $2
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(screener.title)
        .bind(screener.description)
        .bind(screener.filter_criteria)
        .bind(screener.sort_config)
        .bind(screener.display_columns)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(updated_screener)
    }

    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> DbResult<bool> {
        let result = sqlx::query(
            r#"
            DELETE FROM screeners
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.rows_affected() > 0)
    }
}
