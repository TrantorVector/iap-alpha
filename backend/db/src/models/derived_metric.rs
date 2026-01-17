use bigdecimal::BigDecimal;
/// Derived metric model
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Derived metric entity
///
/// Stores pre-computed financial metrics like margins, growth rates, etc.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DerivedMetric {
    pub id: Uuid,
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,

    /// Metric name (e.g., "gross_margin_pct", "yoy_revenue_growth_pct")
    pub metric_name: String,

    /// Metric value
    pub metric_value: Option<BigDecimal>,

    // Audit
    pub created_at: DateTime<Utc>,
}
