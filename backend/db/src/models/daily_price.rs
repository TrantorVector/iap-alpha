/// Daily price model
use chrono::{DateTime, NaiveDate, Utc};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Daily price entity (OHLCV data)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DailyPrice {
    pub id: Uuid,
    pub company_id: Uuid,
    pub price_date: NaiveDate,
    
    // OHLC
    pub open: Option<BigDecimal>,
    pub high: Option<BigDecimal>,
    pub low: Option<BigDecimal>,
    pub close: Option<BigDecimal>,
    pub adjusted_close: Option<BigDecimal>,
    
    // Volume & Corporate Actions
    pub volume: Option<i64>,
    pub dividend_amount: Option<BigDecimal>,
    pub split_coefficient: Option<BigDecimal>,
    
    // Audit
    pub created_at: DateTime<Utc>,
}
