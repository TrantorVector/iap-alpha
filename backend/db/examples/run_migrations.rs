//! Database migration runner
//!
//! This example connects to the database and runs all pending migrations.
//!
//! Usage:
//!   cargo run --example run_migrations
//!
//! Make sure DATABASE_URL environment variable is set, or it will use the default.

use db::{init_pool, run_migrations};

#[tokio::main]
async fn main() {
    // Get database URL from environment or use default
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());

    println!("🔗 Connecting to database...");
    println!("   URL: {}", database_url.replace(":dev@", ":***@"));

    // Initialize connection pool
    let pool = match init_pool(&database_url).await {
        Ok(pool) => {
            println!("✅ Connected successfully");
            pool
        }
        Err(e) => {
            eprintln!("❌ Failed to connect to database: {}", e);
            eprintln!("\nTroubleshooting:");
            eprintln!("  1. Make sure PostgreSQL is running on port 5432");
            eprintln!("  2. Check database credentials (postgres:dev)");
            eprintln!("  3. Verify the database 'irp_dev' exists");
            eprintln!("\nTo create the database manually:");
            eprintln!("  docker exec -it <postgres-container> psql -U postgres");
            eprintln!("  CREATE DATABASE irp_dev;");
            std::process::exit(1);
        }
    };

    println!("\n📦 Running migrations...");
    println!("   Source: backend/db/migrations/");

    // Run migrations
    match run_migrations(&pool).await {
        Ok(_) => {
            println!("✅ Migrations completed successfully");

            // Query to check tables
            let result = match sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'",
            )
            .fetch_one(&pool)
            .await
            {
                Ok(count) => count,
                Err(e) => {
                    eprintln!("Warning: Failed to count tables: {}", e);
                    0
                }
            };

            println!("\n📊 Database Status:");
            println!("   Tables created: {}", result);

            // Check for sample data
            let company_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM companies")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .unwrap_or(0);

            println!("   Sample companies: {}", company_count);
            println!("   Test users: {}", user_count);

            println!("\n🎉 Database setup complete!");
        }
        Err(e) => {
            eprintln!("❌ Migration failed: {}", e);
            eprintln!("\nThis could mean:");
            eprintln!("  1. Migrations have already been run");
            eprintln!("  2. There's a syntax error in the SQL files");
            eprintln!("  3. Database permissions issue");
            std::process::exit(1);
        }
    }
}
