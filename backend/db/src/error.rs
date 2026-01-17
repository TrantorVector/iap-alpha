/// Database error types
use thiserror::Error;

/// Database operation result type
pub type DbResult<T> = Result<T, DbError>;

/// Database errors
#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Duplicate record: {0}")]
    DuplicateError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Optimistic lock error: {0}")]
    OptimisticLockError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

// Convert sqlx errors to DbError
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound("Row not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // Check for unique constraint violations
                if let Some(code) = db_err.code() {
                    if code == "23505" {
                        return DbError::DuplicateError(db_err.message().to_string());
                    }
                }
                DbError::DatabaseError(db_err.message().to_string())
            }
            _ => DbError::QueryError(err.to_string()),
        }
    }
}
