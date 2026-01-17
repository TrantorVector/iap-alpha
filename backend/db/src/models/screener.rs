/// Screener model
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Screener entity
/// 
/// User-defined stock screeners with filter criteria stored as JSONB
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Screener {
    pub id: Uuid,
    pub user_id: Uuid,
    
    // Definition
    pub title: String,
    pub description: Option<String>,
    
    /// Filter criteria as JSON
    /// Example: {"exchanges": ["NASDAQ"], "sectors": ["Technology"], "market_cap": {"min": 1000000000}}
    pub filter_criteria: serde_json::Value,
    
    /// Sort configuration as JSON
    /// Example: {"column": "market_cap", "direction": "desc"}
    pub sort_config: Option<serde_json::Value>,
    
    /// Display columns as JSON array
    /// Example: ["symbol", "name", "market_cap"]
    pub display_columns: Option<serde_json::Value>,
    
    // Ordering
    pub display_order: Option<i32>,
    
    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
