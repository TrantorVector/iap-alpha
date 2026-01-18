use db::{init_pool, run_migrations};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Running migrations on {}...", database_url);

    let pool = init_pool(&database_url).await?;
    run_migrations(&pool).await?;

    println!("Migrations completed successfully!");
    Ok(())
}
