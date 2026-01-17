/// Document models
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Document entity
///
/// Metadata for company documents stored in S3
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Document {
    pub id: Uuid,
    pub company_id: Uuid,

    // Document metadata
    pub document_type: String, // "investor_presentation" | "earnings_call_transcript" | etc.
    pub period_end_date: Option<NaiveDate>,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    pub title: String,

    // Storage
    pub storage_key: Option<String>, // S3 key
    pub source_url: Option<String>,

    // File metadata
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Document {
    pub fn is_available(&self) -> bool {
        self.storage_key.is_some()
    }
}

/// Analysis report entity
///
/// User-uploaded analysis documents (linked to verdicts)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AnalysisReport {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub verdict_history_id: Option<Uuid>,

    // Storage
    pub storage_key: String,
    pub filename: String,

    // Audit
    pub uploaded_at: DateTime<Utc>,
}
