use crate::domain::{Filing, Transcript};
use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait DocumentProvider: Send + Sync {
    async fn get_earnings_transcript(
        &self,
        symbol: &str,
        year: i32,
        quarter: i32,
    ) -> Result<Transcript, AppError>;
    async fn list_sec_filings(
        &self,
        symbol: &str,
        filing_type: &str,
    ) -> Result<Vec<Filing>, AppError>;
}
