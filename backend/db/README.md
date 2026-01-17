# Database Crate

This crate contains the database layer for the Investment Research Platform, including:
- Database models (structs mapping to tables)
- Repository pattern for data access
- Connection pool management
- Migration scripts

## Setup

### 1. Install SQLx CLI

The SQLx CLI tool is required for running migrations and generating offline query data.

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 2. Set up PostgreSQL

Ensure you have PostgreSQL running. You can use Docker:

```bash
docker run -d \
  --name irp-postgres \
  -e POSTGRES_PASSWORD=dev \
  -e POSTGRES_DB=irp_dev \
  -p 5432:5432 \
  postgres:15-alpine
```

### 3. Set Environment Variable

Create a `.env` file in the project root or export the variable:

```bash
export DATABASE_URL="postgres://postgres:dev@localhost:5432/irp_dev"
```

### 4. Run Migrations

```bash
cd backend/db
sqlx migrate run
```

## Usage

### Creating the Database Pool

```rust
use db::init_pool;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = init_pool(&database_url).await?;
    
    // Use the pool for queries
    Ok(())
}
```

### Using Repositories

```rust
use db::repositories::{PgUserRepository, UserRepository};
use db::models::User;

async fn example(pool: sqlx::PgPool) {
    let user_repo = PgUserRepository::new(pool);
    
    // Find user by email
    let user = user_repo
        .find_by_email("test@example.com")
        .await
        .expect("User not found");
    
    println!("Found user: {}", user.username);
}
```

## SQLx Offline Mode

SQLx can verify queries at compile time. To enable this:

1. Ensure database is running with latest migrations
2. Generate query metadata:

```bash
cd backend/db
cargo sqlx prepare
```

This creates `sqlx-data.json` which allows compilation without a database connection.

## Migrations

Migrations are located in `migrations/` directory. To create a new migration:

```bash
sqlx migrate add <migration_name>
```

This creates a new timestamped SQL file in the migrations directory.

### Migration Naming Convention

- `001_initial_schema.sql` - Initial database schema
- `002_seed_data.sql` - Development seed data
- `003_add_xyz_table.sql` - Descriptive name for schema changes

## Development

### Running Tests

```bash
# Requires running PostgreSQL instance
cargo test -p db

# Run only unit tests (no DB required)
cargo test -p db --lib
```

### Checking Compilation

```bash
cargo check -p db
```

## Schema

The database schema is documented in `docs/database-design-v1.md`.

Key tables:
- `users` - User accounts and authentication
- `companies` - Company master data
- `income_statements`, `balance_sheets`, `cash_flow_statements` - Financial data
- `screeners` - User-defined stock screeners
- `verdicts` - Investment analysis verdicts
- `documents` - Company documents (S3 references)

## Troubleshooting

### "Database does not exist" error

Create the database manually:

```bash
createdb irp_dev
# or
psql -U postgres -c "CREATE DATABASE irp_dev;"
```

### Migration conflicts

If migrations are out of sync, you can reset:

```bash
sqlx migrate revert
sqlx migrate run
```

⚠️ **WARNING**: This will delete all data in development!

### Connection timeouts

Increase the connection timeout in `lib.rs`:

```rust
.acquire_timeout(Duration::from_secs(60))
```
