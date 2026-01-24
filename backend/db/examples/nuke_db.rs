use sqlx::postgres::PgPoolOptions;
use sqlx::Executor;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:dev@localhost:5432/irp_dev".to_string());

    println!("💣 Nuking database schema at {}...", database_url);

    // We need to connect to the database to drop schema
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await?;

    pool.execute(
        "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO public;",
    )
    .await?;

    println!("💥 Database schema reset successfully!");
    Ok(())
}
