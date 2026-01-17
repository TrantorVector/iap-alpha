/// Verdict models
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Verdict entity
///
/// User's investment analysis verdict for a company
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Verdict {
    pub id: Uuid,
    pub company_id: Uuid,
    pub user_id: Uuid,

    // Assessment
    pub final_verdict: Option<String>, // "invest" | "pass" | "watchlist"
    pub summary_text: Option<String>,

    /// Strengths as JSON array
    /// Example: ["Strong revenue growth", "Improving margins"]
    pub strengths: Option<serde_json::Value>,

    /// Weaknesses as JSON array
    /// Example: ["High debt levels", "Declining market share"]
    pub weaknesses: Option<serde_json::Value>,

    pub guidance_summary: Option<String>,

    // Version control
    pub lock_version: i32,

    // Audit
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Verdict history entity
///
/// Tracks version history of verdicts
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VerdictHistory {
    pub id: Uuid,
    pub verdict_id: Uuid,
    pub version: i32,

    // Snapshot of verdict at this version
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,

    // Audit
    pub recorded_at: DateTime<Utc>,
}
