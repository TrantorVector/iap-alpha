pub mod company;
pub mod document;
pub mod screener_repository;
pub mod tracker_repository;
/// Database repositories
///
/// This module contains repository pattern implementations for data access.
/// Each repository handles CRUD operations for a specific entity.
pub mod user;
pub mod verdict;

// Re-export commonly used repositories
pub use company::{
    BalanceSheetInsert, CashFlowStatementInsert, CompanyFilters, CompanyRepository,
    DailyPriceInsert, IncomeStatementInsert, Pagination,
};
pub use document::{CreateDocumentParams, DocumentRepository};
pub use screener_repository::{CreateScreener, ScreenerRepository, UpdateScreener};
pub use tracker_repository::{
    Pagination as TrackerPagination, TrackerItem, TrackerRepository, TrackerSummary,
    VerdictFilters as TrackerFilters, VerdictListResult,
};
pub use user::{CreateUserRequest, UserPreferencesUpdate, UserRepository};
pub use verdict::{VerdictCreate, VerdictRepository, VerdictUpdate};
