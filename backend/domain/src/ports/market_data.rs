use crate::domain::{
    BalanceSheet, CashFlowStatement, CompanyOverview, DailyPrice, EarningsEvent, IncomeStatement,
    OutputSize,
};
use crate::error::AppError;
use async_trait::async_trait;

#[async_trait]
pub trait MarketDataProvider: Send + Sync {
    async fn get_company_overview(&self, symbol: &str) -> Result<CompanyOverview, AppError>;
    async fn get_income_statement(&self, symbol: &str) -> Result<Vec<IncomeStatement>, AppError>;
    async fn get_balance_sheet(&self, symbol: &str) -> Result<Vec<BalanceSheet>, AppError>;
    async fn get_cash_flow(&self, symbol: &str) -> Result<Vec<CashFlowStatement>, AppError>;
    async fn get_daily_prices(
        &self,
        symbol: &str,
        output_size: OutputSize,
    ) -> Result<Vec<DailyPrice>, AppError>;
    async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError>;
}
