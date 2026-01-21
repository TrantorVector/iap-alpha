use async_trait::async_trait;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use domain::domain::{
    BalanceSheet, CashFlowStatement, CompanyOverview, DailyPrice, EarningsEvent, IncomeStatement,
    OutputSize,
};
use domain::error::AppError;
use domain::ports::market_data::MarketDataProvider;
use providers::mock::MockMarketDataProvider;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;
use worker::jobs::{EarningsPollingJob, Job, MetricsRecalculationJob, PriceRefreshJob};

async fn setup_db() -> sqlx::PgPool {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());

    PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to DB")
}

async fn clear_db(pool: &sqlx::PgPool) {
    // Order matters for FK
    sqlx::query("DELETE FROM job_runs").execute(pool).await.ok();
    sqlx::query("DELETE FROM derived_metrics")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM daily_prices")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM income_statements")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM balance_sheets")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM cash_flow_statements")
        .execute(pool)
        .await
        .ok();
    sqlx::query("DELETE FROM companies")
        .execute(pool)
        .await
        .ok();
}

async fn seed_company(pool: &sqlx::PgPool, symbol: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO companies (id, symbol, name, exchange, is_active, shares_outstanding) VALUES ($1, $2, $3, $4, true, 1000000)"
    )
    .bind(id)
    .bind(symbol)
    .bind(format!("{} Inc", symbol))
    .bind("NASDAQ")
    .execute(pool)
    .await
    .unwrap();
    id
}

#[tokio::test]
async fn test_earnings_poll_updates_calendar() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    let company_id = seed_company(&pool, "IBM").await;

    let provider = Arc::new(MockMarketDataProvider::new());
    let job = EarningsPollingJob::new(pool.clone(), provider);

    job.run(&pool).await.expect("Job failed");

    let latest_quarter: Option<NaiveDate> =
        sqlx::query_scalar("SELECT latest_quarter FROM companies WHERE id = $1")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(
        latest_quarter.is_some(),
        "latest_quarter should be updated for IBM"
    );
}

#[tokio::test]
async fn test_price_refresh_updates_prices() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    seed_company(&pool, "IBM").await;

    let provider = Arc::new(MockMarketDataProvider::new());
    let job = PriceRefreshJob::new(pool.clone(), provider);

    job.run(&pool).await.expect("Job failed");

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM daily_prices")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(count > 0, "daily_prices should have records");
}

#[tokio::test]
async fn test_price_refresh_updates_market_cap() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    let company_id = seed_company(&pool, "IBM").await;

    let provider = Arc::new(MockMarketDataProvider::new());
    let job = PriceRefreshJob::new(pool.clone(), provider);

    job.run(&pool).await.expect("Job failed");

    let market_cap: Option<i64> =
        sqlx::query_scalar("SELECT market_cap FROM companies WHERE id = $1")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(market_cap.is_some(), "market_cap should be updated");
    assert!(market_cap.unwrap() > 0, "market_cap should be positive");
}

#[tokio::test]
async fn test_metrics_recalc_creates_derived_metrics() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    let company_id = seed_company(&pool, "IBM").await;

    // Seed data needed for metrics
    sqlx::query(
        "INSERT INTO income_statements (id, company_id, period_end_date, period_type, total_revenue, net_income) VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind(Uuid::new_v4())
    .bind(company_id)
    .bind(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
    .bind("annual")
    .bind(BigDecimal::from_str("1000000").unwrap())
    .bind(BigDecimal::from_str("100000").unwrap())
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO daily_prices (company_id, price_date, close) VALUES ($1, $2, $3)")
        .bind(company_id)
        .bind(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap())
        .bind(BigDecimal::from_str("150").unwrap())
        .execute(&pool)
        .await
        .unwrap();

    let job = MetricsRecalculationJob;
    job.run(&pool).await.expect("Job failed");

    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM derived_metrics WHERE company_id = $1")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert!(count > 0, "derived_metrics should be created");
}

#[tokio::test]
async fn test_job_records_success_in_database() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    seed_company(&pool, "IBM").await;

    let provider = Arc::new(MockMarketDataProvider::new());
    let job = EarningsPollingJob::new(pool.clone(), provider);

    job.run(&pool).await.expect("Job failed");

    let status: String =
        sqlx::query_scalar("SELECT status FROM job_runs ORDER BY started_at DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(status, "completed");
}

struct FailingProvider;
#[async_trait]
impl MarketDataProvider for FailingProvider {
    async fn get_company_overview(&self, _symbol: &str) -> Result<CompanyOverview, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
    async fn get_income_statement(&self, _symbol: &str) -> Result<Vec<IncomeStatement>, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
    async fn get_balance_sheet(&self, _symbol: &str) -> Result<Vec<BalanceSheet>, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
    async fn get_cash_flow(&self, _symbol: &str) -> Result<Vec<CashFlowStatement>, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
    async fn get_daily_prices(
        &self,
        _symbol: &str,
        _output_size: OutputSize,
    ) -> Result<Vec<DailyPrice>, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
    async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError> {
        Err(AppError::InternalError("Provider error".into()))
    }
}

#[tokio::test]
async fn test_job_handles_provider_errors_gracefully() {
    let pool = setup_db().await;
    clear_db(&pool).await;
    seed_company(&pool, "IBM").await;

    let provider = Arc::new(FailingProvider);
    let job = EarningsPollingJob::new(pool.clone(), provider);

    job.run(&pool)
        .await
        .expect("Job should return Ok but set status to failed");

    let status: String =
        sqlx::query_scalar("SELECT status FROM job_runs ORDER BY started_at DESC LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(status, "failed");
}
