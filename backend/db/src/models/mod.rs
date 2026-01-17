pub mod company;
pub mod daily_price;
pub mod derived_metric;
pub mod document;
pub mod financials;
pub mod screener;
/// Database models
///
/// This module contains structs that map to database tables.
/// Models use SQLx's FromRow derive macro for automatic deserialization.
pub mod user;
pub mod verdict;

// Re-export commonly used models
pub use company::Company;
pub use daily_price::DailyPrice;
pub use derived_metric::DerivedMetric;
pub use document::{AnalysisReport, Document};
pub use financials::{BalanceSheet, CashFlowStatement, IncomeStatement};
pub use screener::Screener;
pub use user::{RefreshToken, User, UserPreferences};
pub use verdict::{Verdict, VerdictHistory};
