use crate::{DbError, DbResult};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// =============================================================================
// DTOs (Data Transfer Objects)
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictFilters {
    pub verdict_type: Option<Vec<String>>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub sector: Option<Vec<String>>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictListResult {
    pub items: Vec<TrackerItem>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TrackerItem {
    pub company_id: Uuid,
    pub symbol: String,
    pub company_name: String,
    pub exchange: String,
    pub sector: Option<String>,
    pub verdict: String,
    pub verdict_date: DateTime<Utc>,
    pub summary_text: String,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerSummary {
    pub total_analyzed: i64,
    pub invest_count: i64,
    pub pass_count: i64,
    pub watchlist_count: i64,
    pub no_thesis_count: i64,
    pub recent_activity: Vec<RecentActivity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RecentActivity {
    pub company_id: Uuid,
    pub symbol: String,
    pub company_name: String,
    pub verdict: String,
    pub recorded_at: DateTime<Utc>,
}

// =============================================================================
// Tracker Repository
// =============================================================================

pub struct TrackerRepository {
    pool: PgPool,
}

impl TrackerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_verdicts(
        &self,
        user_id: Uuid,
        filters: VerdictFilters,
        pagination: Pagination,
    ) -> DbResult<VerdictListResult> {
        let offset = (pagination.page - 1) * pagination.per_page;

        // Build query
        let verdict_types = filters.verdict_type.as_ref().cloned().unwrap_or_default();
        let sectors = filters.sector.as_ref().cloned().unwrap_or_default();
        let search = filters.search.as_ref().map(|s| format!("%{}%", s.to_lowercase()));

        let mut query_builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            r#"
            SELECT 
                c.id as company_id,
                c.symbol,
                c.name as company_name,
                c.exchange,
                c.sector,
                v.final_verdict as verdict,
                v.updated_at as verdict_date,
                v.summary_text,
                v.lock_version as version
            FROM verdicts v
            JOIN companies c ON v.company_id = c.id
            WHERE v.user_id = "#
        );
        query_builder.push_bind(user_id);

        if !verdict_types.is_empty() {
            query_builder.push(" AND v.final_verdict = ANY(");
            query_builder.push_bind(verdict_types);
            query_builder.push(")");
        }

        if let Some(date_from) = filters.date_from {
            query_builder.push(" AND v.updated_at >= ");
            query_builder.push_bind(date_from.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap());
        }

        if let Some(date_to) = filters.date_to {
            query_builder.push(" AND v.updated_at <= ");
            query_builder.push_bind(date_to.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Utc).unwrap());
        }

        if !sectors.is_empty() {
            query_builder.push(" AND c.sector = ANY(");
            query_builder.push_bind(sectors);
            query_builder.push(")");
        }

        if let Some(search_pattern) = search {
            query_builder.push(" AND (LOWER(c.symbol) LIKE ");
            query_builder.push_bind(search_pattern.clone());
            query_builder.push(" OR LOWER(c.name) LIKE ");
            query_builder.push_bind(search_pattern);
            query_builder.push(")");
        }

        // Clone for count before adding order and limit
        let mut count_query_builder = sqlx::QueryBuilder::<sqlx::Postgres>::new("SELECT COUNT(*) FROM (");
        count_query_builder.push(query_builder.sql());
        // We need to re-bind arguments for the count query if we want to use the same builder logic
        // But sqlx::QueryBuilder doesn't easily allow cloning with binds.
        // Let's use a simpler approach for now.
        
        // Actually, let's just do two separate queries or one with window function.
        // Window function is easier for pagination metadata.
        
        let mut final_query_builder = query_builder;
        final_query_builder.push(" ORDER BY v.updated_at DESC LIMIT ");
        final_query_builder.push_bind(pagination.per_page as i64);
        final_query_builder.push(" OFFSET ");
        final_query_builder.push_bind(offset as i64);

        let items: Vec<TrackerItem> = final_query_builder
            .build_query_as::<TrackerItem>()
            .fetch_all(&self.pool)
            .await
            .map_err(DbError::from)?;

        // Now get count
        let mut count_query_builder = sqlx::QueryBuilder::<sqlx::Postgres>::new(
            "SELECT COUNT(*) FROM verdicts v JOIN companies c ON v.company_id = c.id WHERE v.user_id = "
        );
        count_query_builder.push_bind(user_id);

        if !filters.verdict_type.as_ref().cloned().unwrap_or_default().is_empty() {
            count_query_builder.push(" AND v.final_verdict = ANY(");
            count_query_builder.push_bind(filters.verdict_type.as_ref().cloned().unwrap_or_default());
            count_query_builder.push(")");
        }
        
        if let Some(date_from) = filters.date_from {
            count_query_builder.push(" AND v.updated_at >= ");
            count_query_builder.push_bind(date_from.and_hms_opt(0, 0, 0).unwrap().and_local_timezone(Utc).unwrap());
        }

        if let Some(date_to) = filters.date_to {
            count_query_builder.push(" AND v.updated_at <= ");
            count_query_builder.push_bind(date_to.and_hms_opt(23, 59, 59).unwrap().and_local_timezone(Utc).unwrap());
        }

        if !filters.sector.as_ref().cloned().unwrap_or_default().is_empty() {
            count_query_builder.push(" AND c.sector = ANY(");
            count_query_builder.push_bind(filters.sector.as_ref().cloned().unwrap_or_default());
            count_query_builder.push(")");
        }

        if let Some(search_text) = filters.search {
            let search_pattern = format!("%{}%", search_text.to_lowercase());
            count_query_builder.push(" AND (LOWER(c.symbol) LIKE ");
            count_query_builder.push_bind(search_pattern.clone());
            count_query_builder.push(" OR LOWER(c.name) LIKE ");
            count_query_builder.push_bind(search_pattern);
            count_query_builder.push(")");
        }

        let total: (i64,) = count_query_builder
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(DbError::from)?;

        Ok(VerdictListResult {
            items,
            total: total.0,
            page: pagination.page,
            per_page: pagination.per_page,
        })
    }

    pub async fn get_summary(&self, user_id: Uuid) -> DbResult<TrackerSummary> {
        #[derive(sqlx::FromRow)]
        struct SummaryCounts {
            total_count: i64,
            invest_count: i64,
            pass_count: i64,
            watchlist_count: i64,
            no_thesis_count: i64,
        }

        let counts = sqlx::query_as::<sqlx::Postgres, SummaryCounts>(
            r#"
            SELECT 
                COUNT(*) as total_count,
                COUNT(*) FILTER (WHERE final_verdict = 'invest') as invest_count,
                COUNT(*) FILTER (WHERE final_verdict = 'pass') as pass_count,
                COUNT(*) FILTER (WHERE final_verdict = 'watchlist') as watchlist_count,
                COUNT(*) FILTER (WHERE final_verdict = 'no_thesis') as no_thesis_count
            FROM verdicts
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        let recent_activity = sqlx::query_as::<sqlx::Postgres, RecentActivity>(
            r#"
            SELECT 
                c.id as company_id,
                c.symbol,
                c.name as company_name,
                v.final_verdict as verdict,
                v.updated_at as recorded_at
            FROM verdicts v
            JOIN companies c ON v.company_id = c.id
            WHERE v.user_id = $1
            ORDER BY v.updated_at DESC
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(TrackerSummary {
            total_analyzed: counts.total_count,
            invest_count: counts.invest_count,
            pass_count: counts.pass_count,
            watchlist_count: counts.watchlist_count,
            no_thesis_count: counts.no_thesis_count,
            recent_activity,
        })
    }
}
