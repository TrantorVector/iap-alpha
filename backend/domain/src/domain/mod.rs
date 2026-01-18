use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyOverview {
    pub symbol: String,
    pub name: String,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomeStatement {
    pub period_end_date: chrono::NaiveDate,
    pub revenue: Option<bigdecimal::BigDecimal>,
    pub gross_profit: Option<bigdecimal::BigDecimal>,
    pub operating_income: Option<bigdecimal::BigDecimal>,
    pub net_income: Option<bigdecimal::BigDecimal>,
    pub eps: Option<bigdecimal::BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceSheet {
    pub period_end_date: chrono::NaiveDate,
    pub total_assets: Option<bigdecimal::BigDecimal>,
    pub total_liabilities: Option<bigdecimal::BigDecimal>,
    pub total_equity: Option<bigdecimal::BigDecimal>,
    pub cash_and_equivalents: Option<bigdecimal::BigDecimal>,
    pub short_term_investments: Option<bigdecimal::BigDecimal>,
    pub short_term_debt: Option<bigdecimal::BigDecimal>,
    pub long_term_debt: Option<bigdecimal::BigDecimal>,
    pub net_debt: Option<bigdecimal::BigDecimal>,
    pub common_stock_shares_outstanding: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashFlowStatement {
    pub period_end_date: chrono::NaiveDate,
    pub operating_cash_flow: Option<bigdecimal::BigDecimal>,
    pub capital_expenditures: Option<bigdecimal::BigDecimal>,
    pub free_cash_flow: Option<bigdecimal::BigDecimal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyPrice {
    pub date: chrono::NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OutputSize {
    Compact,
    Full,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EarningsEvent {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transcript {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filing {
    // Add fields
}
