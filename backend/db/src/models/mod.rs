/// Database models
/// 
/// This module contains structs that map to database tables.
/// Models use SQLx's FromRow derive macro for automatic deserialization.

pub mod user;
pub mod company;
pub mod financials;
pub mod daily_price;
pub mod derived_metric;
pub mod screener;
pub mod verdict;
pub mod document;

// Re-export commonly used models
pub use user::{User, UserPreferences, RefreshToken};
pub use company::Company;
pub use financials::{IncomeStatement, BalanceSheet, CashFlowStatement};
pub use daily_price::DailyPrice;
pub use derived_metric::DerivedMetric;
pub use screener::Screener;
pub use verdict::{Verdict, VerdictHistory};
pub use document::{Document, AnalysisReport};
