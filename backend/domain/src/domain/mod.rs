use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyOverview {
    pub symbol: String,
    pub name: String,
    pub description: Option<String>,
    pub exchange: Option<String>,
    pub currency: Option<String>,
    pub country: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_capitalization: Option<i64>,
    pub ebitda: Option<i64>,
    pub pe_ratio: Option<f64>,
    pub peg_ratio: Option<f64>,
    pub book_value: Option<f64>,
    pub dividend_per_share: Option<f64>,
    pub dividend_yield: Option<f64>,
    pub eps: Option<f64>,
    pub revenue_per_share_ttm: Option<f64>,
    pub profit_margin: Option<f64>,
    pub operating_margin_ttm: Option<f64>,
    pub return_on_assets_ttm: Option<f64>,
    pub return_on_equity_ttm: Option<f64>,
    pub revenue_ttm: Option<i64>,
    pub gross_profit_ttm: Option<i64>,
    pub diluted_eps_ttm: Option<f64>,
    pub quarterly_earnings_growth_yoy: Option<f64>,
    pub quarterly_revenue_growth_yoy: Option<f64>,
    pub analyst_target_price: Option<f64>,
    pub trailing_pe: Option<f64>,
    pub forward_pe: Option<f64>,
    pub price_to_sales_ratio_ttm: Option<f64>,
    pub price_to_book_ratio: Option<f64>,
    pub ev_to_revenue: Option<f64>,
    pub ev_to_ebitda: Option<f64>,
    pub beta: Option<f64>,
    pub week_52_high: Option<f64>,
    pub week_52_low: Option<f64>,
    pub day_50_moving_average: Option<f64>,
    pub day_200_moving_average: Option<f64>,
    pub shares_outstanding: Option<i64>,
    pub shares_float: Option<i64>,
    pub percent_insiders: Option<f64>,
    pub percent_institutions: Option<f64>,
    pub dividend_date: Option<chrono::NaiveDate>,
    pub ex_dividend_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EarningsEvent {
    pub symbol: String,
    pub name: String,
    pub report_date: chrono::NaiveDate,
    pub fiscal_date_ending: Option<chrono::NaiveDate>,
    pub estimate: Option<f64>,
    pub currency: Option<String>,
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
pub struct Transcript {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Filing {
    // Add fields
}
