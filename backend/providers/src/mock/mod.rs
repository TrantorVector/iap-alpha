use async_trait::async_trait;
use bytes::Bytes;
use domain::domain::{
    BalanceSheet, CashFlowStatement, CompanyOverview, DailyPrice, EarningsEvent, IncomeStatement,
    OutputSize,
};
use domain::error::AppError;
use domain::ports::market_data::MarketDataProvider;
use domain::ports::storage::ObjectStorage;
use std::time::Duration;

#[derive(Clone, Default)]
pub struct MockMarketDataProvider;

impl MockMarketDataProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn get_company_overview(&self, _symbol: &str) -> Result<CompanyOverview, AppError> {
        Err(AppError::InternalError(
            "Mock implementation pending".into(),
        ))
    }
    async fn get_income_statement(&self, _symbol: &str) -> Result<Vec<IncomeStatement>, AppError> {
        Ok(vec![])
    }
    async fn get_balance_sheet(&self, _symbol: &str) -> Result<Vec<BalanceSheet>, AppError> {
        Ok(vec![])
    }
    async fn get_cash_flow(&self, _symbol: &str) -> Result<Vec<CashFlowStatement>, AppError> {
        Ok(vec![])
    }
    async fn get_daily_prices(
        &self,
        _symbol: &str,
        _output_size: OutputSize,
    ) -> Result<Vec<DailyPrice>, AppError> {
        Ok(vec![])
    }
    async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError> {
        Ok(vec![])
    }
}

#[derive(Clone, Default)]
pub struct MockObjectStorage;

impl MockObjectStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ObjectStorage for MockObjectStorage {
    async fn put_object(
        &self,
        _key: &str,
        _data: Bytes,
        _content_type: &str,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_object(&self, _key: &str) -> Result<Bytes, AppError> {
        Ok(Bytes::new())
    }
    async fn get_presigned_url(
        &self,
        _key: &str,
        _expires_in: Duration,
    ) -> Result<String, AppError> {
        Ok("http://localhost".to_string())
    }
    async fn delete_object(&self, _key: &str) -> Result<(), AppError> {
        Ok(())
    }
}
