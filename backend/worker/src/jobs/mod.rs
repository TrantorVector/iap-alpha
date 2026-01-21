use anyhow::Result;
use async_trait::async_trait;
use sqlx::PgPool;
use tracing::info;

#[async_trait]
pub trait Job: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, pool: &PgPool) -> Result<()>;
}

pub mod earnings_poll;
pub use earnings_poll::EarningsPollingJob;

pub mod price_refresh;
pub use price_refresh::PriceRefreshJob;

pub struct FxRefresh;
#[async_trait]
impl Job for FxRefresh {
    fn name(&self) -> &str {
        "fx_refresh"
    }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running fx_refresh job");
        Ok(())
    }
}

pub struct DocumentRefresh;
#[async_trait]
impl Job for DocumentRefresh {
    fn name(&self) -> &str {
        "document_refresh"
    }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running document_refresh job");
        Ok(())
    }
}

pub mod metrics_recalc;
pub use metrics_recalc::MetricsRecalculationJob;
