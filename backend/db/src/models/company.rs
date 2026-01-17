/// Company model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Company entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Company {
    pub id: Uuid,
    pub symbol: String,
    pub exchange: String,
    pub name: String,
    pub sector_id: Option<Uuid>,
    pub industry: Option<String>,
    pub country: Option<String>,
    pub market_cap: Option<i64>,
    pub currency: Option<String>,
    pub fiscal_year_end_month: Option<i32>,
    pub description: Option<String>,
    pub cik: Option<String>,
    pub address: Option<String>,
    pub latest_quarter: Option<chrono::NaiveDate>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
