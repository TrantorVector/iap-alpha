use domain::domain::*;
use domain::error::AppError;
use domain::ports::market_data::MarketDataProvider;
use async_trait::async_trait;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AlphaVantageClient {
    api_key: String,
}

impl AlphaVantageClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

#[async_trait]
impl MarketDataProvider for AlphaVantageClient {
    async fn get_company_overview(&self, _symbol: &str) -> Result<CompanyOverview, AppError> {
        unimplemented!()
    }
    async fn get_income_statement(&self, _symbol: &str) -> Result<Vec<IncomeStatement>, AppError> {
        unimplemented!()
    }
    async fn get_balance_sheet(&self, _symbol: &str) -> Result<Vec<BalanceSheet>, AppError> {
        unimplemented!()
    }
    async fn get_cash_flow(&self, _symbol: &str) -> Result<Vec<CashFlowStatement>, AppError> {
        unimplemented!()
    }
    async fn get_daily_prices(
        &self,
        _symbol: &str,
        _output_size: OutputSize,
    ) -> Result<Vec<DailyPrice>, AppError> {
        unimplemented!()
    }
    async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError> {
        unimplemented!()
    }
}
