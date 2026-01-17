/// Database repositories
/// 
/// This module contains repository pattern implementations for data access.
/// Each repository handles CRUD operations for a specific entity.

pub mod user;
pub mod company;
pub mod verdict;

// Re-export commonly used repositories
pub use user::{CreateUserRequest, UserPreferencesUpdate, UserRepository};
pub use company::{
    BalanceSheetInsert, CashFlowStatementInsert, CompanyFilters, CompanyRepository,
    DailyPriceInsert, IncomeStatementInsert, Pagination,
};
pub use verdict::{VerdictCreate, VerdictRepository, VerdictUpdate};
