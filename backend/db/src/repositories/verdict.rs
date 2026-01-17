/// Verdict repository with optimistic locking support
use crate::models::{Verdict, VerdictHistory};
use crate::{DbError, DbResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// =============================================================================
// DTOs (Data Transfer Objects)
// =============================================================================

/// Verdict creation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictCreate {
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,
    pub strengths: Option<serde_json::Value>,
    pub weaknesses: Option<serde_json::Value>,
    pub guidance_summary: Option<String>,
}

/// Verdict update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictUpdate {
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,
    pub strengths: Option<serde_json::Value>,
    pub weaknesses: Option<serde_json::Value>,
    pub guidance_summary: Option<String>,
}

// =============================================================================
// Verdict Repository
// =============================================================================

/// Verdict repository
pub struct VerdictRepository {
    pool: PgPool,
}

impl VerdictRepository {
    /// Create a new verdict repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // Query Methods
    // =========================================================================

    /// Find verdict by company ID and user ID
    pub async fn find_by_company(
        &self,
        company_id: Uuid,
        user_id: Uuid,
    ) -> DbResult<Option<Verdict>> {
        let verdict = sqlx::query_as::<_, Verdict>(
            r#"
            SELECT id, company_id, user_id, final_verdict, summary_text,
                   strengths, weaknesses, guidance_summary, lock_version,
                   created_at, updated_at
            FROM verdicts
            WHERE company_id = $1 AND user_id = $2
            "#,
        )
        .bind(company_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(verdict)
    }

    /// Find verdict by ID
    pub async fn find_by_id(&self, verdict_id: Uuid) -> DbResult<Option<Verdict>> {
        let verdict = sqlx::query_as::<_, Verdict>(
            r#"
            SELECT id, company_id, user_id, final_verdict, summary_text,
                   strengths, weaknesses, guidance_summary, lock_version,
                   created_at, updated_at
            FROM verdicts
            WHERE id = $1
            "#,
        )
        .bind(verdict_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(verdict)
    }

    // =========================================================================
    // Optimistic Locking Update
    // =========================================================================

    /// Update verdict with optimistic locking
    ///
    /// This method implements optimistic locking by checking the lock_version.
    /// If the version matches, it updates the verdict and increments lock_version.
    /// If the version doesn't match, it returns an OptimisticLockError.
    ///
    /// # Arguments
    /// * `verdict_id` - The verdict ID to update
    /// * `update` - The update data
    /// * `expected_version` - The expected lock version
    ///
    /// # Returns
    /// * `Ok(Verdict)` - The updated verdict with new lock_version
    /// * `Err(DbError::OptimisticLockError)` - If version mismatch detected
    pub async fn update_with_lock(
        &self,
        verdict_id: Uuid,
        update: VerdictUpdate,
        expected_version: i32,
    ) -> DbResult<Verdict> {
        let now = Utc::now();

        // Attempt to update with optimistic lock check
        let result = sqlx::query_as::<_, Verdict>(
            r#"
            UPDATE verdicts
            SET final_verdict = COALESCE($1, final_verdict),
                summary_text = COALESCE($2, summary_text),
                strengths = COALESCE($3, strengths),
                weaknesses = COALESCE($4, weaknesses),
                guidance_summary = COALESCE($5, guidance_summary),
                lock_version = lock_version + 1,
                updated_at = $6
            WHERE id = $7 AND lock_version = $8
            RETURNING id, company_id, user_id, final_verdict, summary_text,
                      strengths, weaknesses, guidance_summary, lock_version,
                      created_at, updated_at
            "#,
        )
        .bind(&update.final_verdict)
        .bind(&update.summary_text)
        .bind(&update.strengths)
        .bind(&update.weaknesses)
        .bind(&update.guidance_summary)
        .bind(now)
        .bind(verdict_id)
        .bind(expected_version)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        match result {
            Some(verdict) => Ok(verdict),
            None => {
                // Fetch current version to provide helpful error message
                let current = self.find_by_id(verdict_id).await?;

                match current {
                    Some(v) => Err(DbError::OptimisticLockError(format!(
                        "Version mismatch: expected {}, current {}",
                        expected_version, v.lock_version
                    ))),
                    None => Err(DbError::NotFound(format!(
                        "Verdict not found: {}",
                        verdict_id
                    ))),
                }
            }
        }
    }

    // =========================================================================
    // Insert Method
    // =========================================================================

    /// Create a new verdict
    pub async fn create(
        &self,
        company_id: Uuid,
        user_id: Uuid,
        verdict: VerdictCreate,
    ) -> DbResult<Verdict> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let created = sqlx::query_as::<_, Verdict>(
            r#"
            INSERT INTO verdicts (
                id, company_id, user_id, final_verdict, summary_text,
                strengths, weaknesses, guidance_summary, lock_version,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 0, $9, $10)
            RETURNING id, company_id, user_id, final_verdict, summary_text,
                      strengths, weaknesses, guidance_summary, lock_version,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(company_id)
        .bind(user_id)
        .bind(&verdict.final_verdict)
        .bind(&verdict.summary_text)
        .bind(&verdict.strengths)
        .bind(&verdict.weaknesses)
        .bind(&verdict.guidance_summary)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(created)
    }

    // =========================================================================
    // History Methods
    // =========================================================================

    /// Create a history snapshot of the current verdict state
    ///
    /// This should be called before any update to preserve the previous state
    pub async fn create_history_snapshot(&self, verdict_id: Uuid) -> DbResult<VerdictHistory> {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let history = sqlx::query_as::<_, VerdictHistory>(
            r#"
            INSERT INTO verdict_history (
                id, verdict_id, version, final_verdict, summary_text, recorded_at
            )
            SELECT $1, id, lock_version, final_verdict, summary_text, $2
            FROM verdicts
            WHERE id = $3
            RETURNING id, verdict_id, version, final_verdict, summary_text, recorded_at
            "#,
        )
        .bind(id)
        .bind(now)
        .bind(verdict_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(history)
    }

    /// Get all history snapshots for a verdict
    ///
    /// Returns snapshots ordered by version (oldest to newest)
    pub async fn get_history(&self, verdict_id: Uuid) -> DbResult<Vec<VerdictHistory>> {
        let history = sqlx::query_as::<_, VerdictHistory>(
            r#"
            SELECT id, verdict_id, version, final_verdict, summary_text, recorded_at
            FROM verdict_history
            WHERE verdict_id = $1
            ORDER BY version ASC
            "#,
        )
        .bind(verdict_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(history)
    }

    /// Get history count for a verdict
    pub async fn get_history_count(&self, verdict_id: Uuid) -> DbResult<i64> {
        let result: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) as count
            FROM verdict_history
            WHERE verdict_id = $1
            "#,
        )
        .bind(verdict_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(result.0)
    }

    /// Delete a verdict and its history
    ///
    /// This will cascade delete to verdict_history and analysis_reports
    pub async fn delete(&self, verdict_id: Uuid) -> DbResult<()> {
        let result = sqlx::query(
            r#"
            DELETE FROM verdicts
            WHERE id = $1
            "#,
        )
        .bind(verdict_id)
        .execute(&self.pool)
        .await
        .map_err(DbError::from)?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!(
                "Verdict not found: {}",
                verdict_id
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL instance
    async fn test_optimistic_locking() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());

        let pool = crate::init_pool(&database_url).await.unwrap();
        let repo = VerdictRepository::new(pool);

        // This test would require setting up test data
        // Just showing the structure
    }

    #[tokio::test]
    #[ignore]
    async fn test_version_mismatch_error() {
        // Test that updating with wrong version fails
        // and returns OptimisticLockError
    }
}
