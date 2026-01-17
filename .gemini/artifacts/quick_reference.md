# Database Quick Reference

**Last Updated**: January 17, 2026  
**Database**: irp_dev  
**Status**: ✅ Operational

---

## 🔗 Connection Information

### Connection String
```
postgres://postgres:dev@localhost:5432/irp_dev
```

### Environment Variable
```bash
export DATABASE_URL="postgres://postgres:dev@localhost:5432/irp_dev"
```

---

## 🛠️ Common Commands

### Run Migrations
```bash
cd backend
cargo run --example run_migrations
```

### Check Database Status
```bash
# Using the migration runner (recommended)
cd backend
cargo run --example run_migrations
```

### Compile Database Crate
```bash
cd backend
cargo check -p db
```

---

## 🗄️ Database Schema

### Tables Created (29 total)

#### User & Auth (3 tables)
- `users`
- `user_preferences`
- `refresh_tokens`

#### Reference Data (5 tables)
- `currencies`
- `exchanges`
- `sectors`
- `market_holidays`

#### Company Data (2 tables)
- `companies`
- `fx_rates`

#### Financial Statements (4 tables)
- `income_statements`
- `balance_sheets`
- `cash_flow_statements`
- `daily_prices` (+ 4 partitions)

#### Analysis (5 tables)
- `derived_metrics`
- `screeners`
- `verdicts`
- `verdict_history`

#### Documents (2 tables)
- `documents`
- `analysis_reports`

#### System (3 tables)
- `api_cache`
- `background_jobs`
- `_sqlx_migrations`

---

## 📊 Seed Data

### Test User
- **Username**: `testuser`
- **Email**: `test@example.com`
- **Password**: `password123`

### Sample Companies (5)
1. **AAPL** - Apple Inc. (NASDAQ)
2. **MSFT** - Microsoft Corp. (NASDAQ)
3. **JPM** - JPMorgan Chase (NYSE)
4. **JNJ** - Johnson & Johnson (NYSE)
5. **TSLA** - Tesla Inc. (NASDAQ)

### Financial Data
- **Apple (AAPL)**: 4 quarters of 2024 data
  - Q1 2024: March 31, 2024
  - Q2 2024: June 30, 2024
  - Q3 2024: September 30, 2024
  - Q4 2024: December 31, 2024

---

## 🔍 Useful Queries

### Count All Tables
```sql
SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public';
-- Expected: 29
```

### List All Tables
```sql
SELECT table_name FROM information_schema.tables 
WHERE table_schema = 'public' 
ORDER BY table_name;
```

### Check Companies
```sql
SELECT symbol, name, exchange_code FROM companies;
-- Expected: 5 rows
```

### Check Test User
```sql
SELECT username, email, created_at FROM users WHERE username = 'testuser';
```

### Apple Financial Data
```sql
-- Income statements
SELECT period_end_date, total_revenue, net_income 
FROM income_statements 
WHERE company_id = (SELECT id FROM companies WHERE symbol = 'AAPL')
ORDER BY period_end_date DESC;

-- Balance sheets
SELECT period_end_date, total_assets, total_equity 
FROM balance_sheets 
WHERE company_id = (SELECT id FROM companies WHERE symbol = 'AAPL')
ORDER BY period_end_date DESC;
```

### Full-Text Search
```sql
-- Search for companies
SELECT symbol, name 
FROM companies 
WHERE search_vector @@ plainto_tsquery('english', 'apple');
```

### Check Partitions
```sql
-- View daily_prices partitions
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size
FROM pg_tables
WHERE tablename LIKE 'daily_prices%'
ORDER BY tablename;
```

---

## 📦 Repository Usage Examples

### UserRepository

```rust
use db::{init_pool, repositories::UserRepository, models::CreateUserRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_pool("postgres://postgres:dev@localhost:5432/irp_dev").await?;
    let user_repo = UserRepository::new(&pool);
    
    // Find user by username
    let user = user_repo.find_by_username("testuser").await?;
    
    // Create new user
    let request = CreateUserRequest {
        username: "newuser".to_string(),
        email: "new@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        timezone: Some("America/New_York".to_string()),
    };
    let new_user = user_repo.create(request).await?;
    
    Ok(())
}
```

### CompanyRepository

```rust
use db::{init_pool, repositories::CompanyRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_pool("postgres://postgres:dev@localhost:5432/irp_dev").await?;
    let company_repo = CompanyRepository::new(&pool);
    
    // Find company by symbol
    let company = company_repo.find_by_symbol("AAPL", "NASDAQ").await?;
    
    // Get income statements
    let statements = company_repo
        .get_income_statements(company.unwrap().id, "quarterly", 4)
        .await?;
    
    // Search companies
    let results = company_repo.search("tech", 10).await?;
    
    Ok(())
}
```

### VerdictRepository

```rust
use db::{init_pool, repositories::VerdictRepository, models::{VerdictCreate, VerdictUpdate}};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_pool("postgres://postgres:dev@localhost:5432/irp_dev").await?;
    let verdict_repo = VerdictRepository::new(&pool);
    
    // Create verdict
    let create = VerdictCreate {
        final_verdict: "BUY".to_string(),
        summary_text: "Strong financials".to_string(),
        // ... other fields
    };
    let verdict = verdict_repo.create(company_id, user_id, create).await?;
    
    // Update with optimistic locking
    let update = VerdictUpdate {
        final_verdict: Some("HOLD".to_string()),
        // ... other fields
    };
    let updated = verdict_repo
        .update_with_lock(verdict.id, update, verdict.lock_version)
        .await?;
    
    Ok(())
}
```

---

## 🔧 Troubleshooting

### Can't Connect to Database
```bash
# Check if PostgreSQL is running
ss -tlnp | grep 5432

# Check Docker container
sudo docker ps | grep postgres

# Restart container if needed
sudo docker restart <container-name>
```

### Migration Errors
```bash
# Check migration status
cd backend/db
sqlx migrate info --database-url postgres://postgres:dev@localhost:5432/irp_dev

# Revert last migration
sqlx migrate revert --database-url postgres://postgres:dev@localhost:5432/irp_dev

# Reset and re-run (⚠️ DESTRUCTIVE)
# This will drop all tables and re-create them
sqlx database drop --database-url postgres://postgres:dev@localhost:5432/irp_dev
sqlx database create --database-url postgres://postgres:dev@localhost:5432/irp_dev
cd backend && cargo run --example run_migrations
```

### Compilation Errors
```bash
# Make sure to run from backend directory
cd /home/preetham/Documents/iap-alpha/backend
cargo check -p db

# Clean and rebuild if needed
cargo clean
cargo check -p db
```

---

## 📚 Related Documentation

- **Build Plan**: `docs/build-plan-v3/04-database-foundation.md`
- **Database Design**: `docs/database-design-v1.md`
- **Architecture**: `docs/architecture-design-v3.md`
- **README**: `backend/db/README.md`

---

**Last Verified**: January 17, 2026  
**Status**: All systems operational ✅
