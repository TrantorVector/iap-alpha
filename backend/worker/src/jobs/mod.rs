use async_trait::async_trait;
use sqlx::PgPool;
use anyhow::Result;
use tracing::info;

#[async_trait]
pub trait Job: Send + Sync {
    fn name(&self) -> &str;
    async fn run(&self, pool: &PgPool) -> Result<()>;
}

pub struct EarningsPoll;
#[async_trait]
impl Job for EarningsPoll {
    fn name(&self) -> &str { "earnings_poll" }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running earnings_poll job");
        Ok(())
    }
}

pub struct PriceRefresh;
#[async_trait]
impl Job for PriceRefresh {
    fn name(&self) -> &str { "price_refresh" }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running price_refresh job");
        Ok(())
    }
}

pub struct FxRefresh;
#[async_trait]
impl Job for FxRefresh {
    fn name(&self) -> &str { "fx_refresh" }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running fx_refresh job");
        Ok(())
    }
}

pub struct DocumentRefresh;
#[async_trait]
impl Job for DocumentRefresh {
    fn name(&self) -> &str { "document_refresh" }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running document_refresh job");
        Ok(())
    }
}

pub struct MetricsRecalc;
#[async_trait]
impl Job for MetricsRecalc {
    fn name(&self) -> &str { "metrics_recalc" }
    async fn run(&self, _pool: &PgPool) -> Result<()> {
        info!("Running metrics_recalc job");
        Ok(())
    }
}
