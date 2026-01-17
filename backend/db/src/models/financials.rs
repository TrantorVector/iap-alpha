/// Financial statement models
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Income statement entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IncomeStatement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    
    // Revenue & Profit
    pub total_revenue: Option<BigDecimal>,
    pub cost_of_revenue: Option<BigDecimal>,
    pub gross_profit: Option<BigDecimal>,
    
    // Operating
    pub operating_expenses: Option<BigDecimal>,
    pub operating_income: Option<BigDecimal>,
    
    // Net income
    pub interest_income: Option<BigDecimal>,
    pub interest_expense: Option<BigDecimal>,
    pub income_before_tax: Option<BigDecimal>,
    pub income_tax_expense: Option<BigDecimal>,
    pub net_income: Option<BigDecimal>,
    
    // Non-cash items
    pub depreciation_amortization: Option<BigDecimal>,
    pub ebit: Option<BigDecimal>,
    pub ebitda: Option<BigDecimal>,
    
    // Per share metrics
    pub basic_eps: Option<BigDecimal>,
    pub diluted_eps: Option<BigDecimal>,
    pub shares_outstanding: Option<i64>,
    
    // Audit
    pub created_at: DateTime<Utc>,
}

/// Balance sheet entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BalanceSheet {
    pub id: Uuid,
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    
    // Assets
    pub total_assets: Option<BigDecimal>,
    pub current_assets: Option<BigDecimal>,
    pub cash_and_equivalents: Option<BigDecimal>,
    pub short_term_investments: Option<BigDecimal>,
    pub inventory: Option<BigDecimal>,
    pub accounts_receivable: Option<BigDecimal>,
    pub non_current_assets: Option<BigDecimal>,
    pub property_plant_equipment: Option<BigDecimal>,
    pub goodwill: Option<BigDecimal>,
    pub intangible_assets: Option<BigDecimal>,
    
    // Liabilities
    pub total_liabilities: Option<BigDecimal>,
    pub current_liabilities: Option<BigDecimal>,
    pub accounts_payable: Option<BigDecimal>,
    pub short_term_debt: Option<BigDecimal>,
    pub non_current_liabilities: Option<BigDecimal>,
    pub long_term_debt: Option<BigDecimal>,
    
    // Equity
    pub total_equity: Option<BigDecimal>,
    pub retained_earnings: Option<BigDecimal>,
    pub common_stock: Option<BigDecimal>,
    
    // Computed fields (from database)
    pub total_debt: Option<BigDecimal>,
    pub net_debt: Option<BigDecimal>,
    
    // Audit
    pub created_at: DateTime<Utc>,
}

/// Cash flow statement entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CashFlowStatement {
    pub id: Uuid,
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    
    // Operating activities
    pub operating_cash_flow: Option<BigDecimal>,
    pub net_income: Option<BigDecimal>,
    pub depreciation_depletion: Option<BigDecimal>,
    pub change_in_receivables: Option<BigDecimal>,
    pub change_in_inventory: Option<BigDecimal>,
    pub change_in_payables: Option<BigDecimal>,
    
    // Investing activities
    pub investing_cash_flow: Option<BigDecimal>,
    pub capital_expenditures: Option<BigDecimal>,
    pub investments: Option<BigDecimal>,
    
    // Financing activities
    pub financing_cash_flow: Option<BigDecimal>,
    pub dividend_payout: Option<BigDecimal>,
    pub stock_repurchase: Option<BigDecimal>,
    pub debt_repayment: Option<BigDecimal>,
    
    // Computed fields (from database)
    pub free_cash_flow: Option<BigDecimal>,
    
    // Audit
    pub created_at: DateTime<Utc>,
}
