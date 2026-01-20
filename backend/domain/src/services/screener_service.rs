use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterCriteria {
    pub exchanges: Option<Vec<String>>,
    pub sectors: Option<Vec<String>>,
    pub market_cap_min: Option<f64>,
    pub market_cap_max: Option<f64>,
    pub momentum_1m_min: Option<f64>,
    pub momentum_3m_min: Option<f64>,
    pub momentum_6m_min: Option<f64>,
    pub has_verdict: Option<bool>,
    pub verdict_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ScreenerResult {
    pub company_id: Uuid,
    pub symbol: String,
    pub company_name: String,
    pub exchange: String,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: f64,
    // market_cap_formatted is computed or can be done in SQL, keeping it simple here or assuming application layer formatting
    // If strict requirement implies it comes from DB, we might need a SQL function or backend logic.
    // For now, let's calculate it or leave it as a string field if the view provides it.
    // Given the prompt asks for it in the struct, I'll add the field but we might need to compute it.
    #[sqlx(default)] 
    pub market_cap_formatted: String, 
    pub momentum_1m: Option<f64>,
    pub momentum_3m: Option<f64>,
    pub momentum_6m: Option<f64>,
    pub revenue_yoy_growth: Option<f64>,
    pub operating_margin: Option<f64>,
    pub verdict: Option<String>,
    pub last_analyzed: Option<DateTime<Utc>>,
    pub guidance_summary: Option<String>,
}

pub struct ScreenerService {
    pool: PgPool,
}

impl ScreenerService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn execute(&self, criteria: FilterCriteria) -> Result<Vec<ScreenerResult>, AppError> {
        let mut query_builder = Self::build_query(&criteria);
        let query = query_builder.build_query_as::<ScreenerResult>();
        
        let mut results = query.fetch_all(&self.pool)
            .await
            .map_err(AppError::DatabaseError)?;

        // Post-processing for formatting
        for result in &mut results {
            result.market_cap_formatted = format_market_cap(result.market_cap);
        }

        Ok(results)
    }

    fn build_query(criteria: &FilterCriteria) -> QueryBuilder<'static, Postgres> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT 
                c.id as company_id,
                c.symbol,
                c.name as company_name,
                c.exchange,
                c.sector,
                c.industry,
                m.market_cap,
                -- format market cap in application code usually, but we can placeholder it here
                '' as market_cap_formatted,
                m.momentum_1m,
                m.momentum_3m,
                m.momentum_6m,
                f.revenue_yoy_growth,
                f.operating_margin,
                a.verdict,
                a.created_at as last_analyzed,
                a.summary as guidance_summary
            FROM companies c
            LEFT JOIN market_data m ON c.id = m.company_id
            LEFT JOIN financial_metrics f ON c.id = f.company_id
                AND f.period = 'TTM' -- Assuming TTM for screener
            LEFT JOIN analysis_results a ON c.id = a.company_id
                AND a.is_latest = true
            WHERE 1=1
            "#
        );

        if let Some(exchanges) = &criteria.exchanges {
            if !exchanges.is_empty() {
                query_builder.push(" AND c.exchange = ANY(");
                query_builder.push_bind(exchanges.clone());
                query_builder.push(")");
            }
        }

        if let Some(sectors) = &criteria.sectors {
            if !sectors.is_empty() {
                query_builder.push(" AND c.sector = ANY(");
                query_builder.push_bind(sectors.clone());
                query_builder.push(")");
            }
        }

        if let Some(min) = criteria.market_cap_min {
            query_builder.push(" AND m.market_cap >= ");
            query_builder.push_bind(min);
        }

        if let Some(max) = criteria.market_cap_max {
            query_builder.push(" AND m.market_cap <= ");
            query_builder.push_bind(max);
        }

        if let Some(min) = criteria.momentum_1m_min {
            query_builder.push(" AND m.momentum_1m >= ");
            query_builder.push_bind(min);
        }

        if let Some(min) = criteria.momentum_3m_min {
            query_builder.push(" AND m.momentum_3m >= ");
            query_builder.push_bind(min);
        }

        if let Some(min) = criteria.momentum_6m_min {
            query_builder.push(" AND m.momentum_6m >= ");
            query_builder.push_bind(min);
        }

        if let Some(has_verdict) = criteria.has_verdict {
            if has_verdict {
                query_builder.push(" AND a.verdict IS NOT NULL");
            } else {
                query_builder.push(" AND a.verdict IS NULL");
            }
        }

        if let Some(verdicts) = &criteria.verdict_types {
            if !verdicts.is_empty() {
                query_builder.push(" AND a.verdict = ANY(");
                query_builder.push_bind(verdicts.clone());
                query_builder.push(")");
            }
        }

        query_builder.push(" ORDER BY m.market_cap DESC LIMIT 100");
        query_builder
    }
}

fn format_market_cap(val: f64) -> String {
    if val >= 1_000_000_000.0 {
        format!("{:.2}B", val / 1_000_000_000.0)
    } else if val >= 1_000_000.0 {
        format!("{:.2}M", val / 1_000_000.0)
    } else {
        format!("{:.2}", val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_building_market_cap_and_exchange() {
        let criteria = FilterCriteria {
            exchanges: Some(vec!["NASDAQ".to_string(), "NYSE".to_string()]),
            sectors: None,
            market_cap_min: Some(1_000_000_000.0),
            market_cap_max: None,
            momentum_1m_min: None,
            momentum_3m_min: None,
            momentum_6m_min: None,
            has_verdict: None,
            verdict_types: None,
        };

        let query_builder = ScreenerService::build_query(&criteria);
        let sql = query_builder.sql();

        assert!(sql.contains("AND c.exchange = ANY("));
        assert!(sql.contains("AND m.market_cap >="));
        assert!(sql.contains("ORDER BY m.market_cap DESC"));
    }
}
