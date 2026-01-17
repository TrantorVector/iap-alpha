// Database crate for Investment Research Platform
// Provides database models, repositories, and connection pooling

pub mod models;
pub mod repositories;
mod error;

// Re-export common types
pub use error::{DbError, DbResult};
pub use sqlx::{PgPool, postgres::PgPoolOptions};
pub use uuid::Uuid;

use std::time::Duration;

/// Initialize database connection pool
///
/// # Arguments
/// * `database_url` - PostgreSQL connection string (e.g., "postgres://user:pass@host/db")
///
/// # Returns
/// * `Result<PgPool>` - Configured connection pool
///
/// # Example
/// ```no_run
/// use db::init_pool;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let pool = init_pool("postgres://localhost/mydb").await?;
///     Ok(())
/// }
/// ```
pub async fn init_pool(database_url: &str) -> DbResult<PgPool> {
    // Create pool with configuration
    let pool = PgPoolOptions::new()
        .max_connections(5) // Limit connections for development
        .acquire_timeout(Duration::from_secs(30)) // 30 second timeout
        .connect(database_url)
        .await
        .map_err(|e| DbError::ConnectionError(e.to_string()))?;

    // Test connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| DbError::QueryError(e.to_string()))?;

    Ok(pool)
}

/// Run database migrations
///
/// # Arguments
/// * `pool` - Database connection pool
///
/// # Returns
/// * `Result<()>` - Success or error
pub async fn run_migrations(pool: &PgPool) -> DbResult<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| DbError::MigrationError(e.to_string()))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires running PostgreSQL instance
    async fn test_pool_initialization() {
        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());
        
        let result = init_pool(&database_url).await;
        assert!(result.is_ok());
    }
}
