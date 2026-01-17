use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CompanyOverview {
    pub symbol: String,
    pub name: String,
    // Add other fields as needed
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomeStatement {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BalanceSheet {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CashFlowStatement {
    // Add fields
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DailyPrice {
    // Add fields
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
