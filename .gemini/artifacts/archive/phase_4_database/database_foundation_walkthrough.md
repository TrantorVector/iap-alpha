# Database Foundation - Session Walkthrough

**Session Date**: January 17, 2026  
**Build Plan Section**: Step 4 - Database Foundation  
**Status**: Phase 4.1-4.3 Complete ✅

---

## 🎯 Session Objectives & Completion Status

### ✅ Completed
1. **Database Schema & Migrations** - Created comprehensive PostgreSQL schema
2. **Seed Data** - Development data for 5 companies with full financial statements
3. **SQLx Integration** - Configured with BigDecimal support
4. **Database Models** - 13 models covering all entities
5. **Repository Layer** - 3 repositories with 38+ methods
   - User Repository (authentication & preferences)
   - Company Repository (companies & financials)
   - Verdict Repository (optimistic locking)

### ⏳ Pending (Next Session)
1. **Phase 4.4**: Git Checkpoint & Documentation
2. **Phase 4.5**: PostgreSQL Setup (Docker or local)
3. **Phase 4.6**: Run Migrations
4. **Phase 4.7**: Verify with Test Queries

---

## 📊 Database Schema Overview

### Schema Location
- **Migrations**: `backend/db/migrations/`
  - `001_initial_schema.sql` (1,389 lines) - Complete schema
  - `002_seed_data.sql` (639 lines) - Dev seed data

### Core Tables (25+ tables)

#### **User & Authentication**
- `users` - User accounts with timezone & preferences
- `user_preferences` - UI preferences (JSONB for flexible config)
- `refresh_tokens` - JWT refresh tokens with revocation support

#### **Company & Reference Data**
- `companies` - Company master data with full-text search
- `exchanges` - Exchange metadata with trading_days JSON
- `sectors` - Industry sectors with GICS codes
- `market_holidays` - Exchange holiday calendar

#### **Financial Statements** (partitioned by year)
- `income_statements` - Revenue, profit, EPS data
- `balance_sheets` - Assets, liabilities, equity (with computed fields)
- `cash_flow_statements` - Operating/Investing/Financing cash flows
- `daily_prices` - OHLCV data (partitioned by year, 2024-2027 partitions created)

#### **Derived Metrics & Analysis**
- `derived_metrics` - Pre-computed financial ratios
- `screeners` - User-defined stock filters (JSONB criteria)
- `verdicts` - Investment analysis with optimistic locking
- `verdict_history` - Version tracking for verdicts

#### **Documents**
- `documents` - S3 metadata for company documents
- `analysis_reports` - User-uploaded analysis files

#### **System Tables**
- `api_cache` - Alpha Vantage API cache
- `background_jobs` - Job queue for data fetching

### Key Schema Features

**1. Partitioning**
```sql
-- daily_prices partitioned by year for performance
CREATE TABLE daily_prices_2024 PARTITION OF daily_prices 
    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');
-- Partitions: 2024, 2025, 2026, 2027
```

**2. Full-Text Search**
```sql
-- Companies have generated search_vector column
ALTER TABLE companies ADD COLUMN search_vector tsvector 
    GENERATED ALWAYS AS (
        setweight(to_tsvector('english', name), 'A') ||
        setweight(to_tsvector('english', symbol), 'B')
    ) STORED;
CREATE INDEX idx_companies_search ON companies USING GIN (search_vector);
```

**3. Computed Columns**
```sql
-- Balance sheet computed fields
total_debt = (COALESCE(short_term_debt, 0) + COALESCE(long_term_debt, 0))
net_debt = (total_debt - COALESCE(cash_and_equivalents, 0))

-- Cash flow computed field
free_cash_flow = (COALESCE(operating_cash_flow, 0) - COALESCE(capital_expenditures, 0))
```

**4. Audit Triggers**
```sql
-- Auto-update updated_at timestamp
CREATE TRIGGER update_companies_updated_at 
    BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
```

**5. Optimistic Locking**
```sql
-- Verdicts table has lock_version for concurrent updates
lock_version INTEGER NOT NULL DEFAULT 0
```

---

## 🏗️ SQLx Integration

### Cargo.toml Configuration
```toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json", "bigdecimal"] }
bigdecimal = { version = "0.3", features = ["serde"] }
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
thiserror = "1.0"
```

### Key Decision: BigDecimal vs Decimal
- **Decision**: Use `bigdecimal` v0.3 (not `rust_decimal`)
- **Reason**: SQLx 0.7 has built-in support for `bigdecimal` via the `bigdecimal` feature
- **Impact**: All financial amounts use `bigdecimal::BigDecimal` type

### Environment Configuration
**File**: `backend/db/.env`
```
DATABASE_URL=postgres://postgres:dev@localhost:5432/irp_dev
```

### Query Approach
- **Runtime verification**: Using `sqlx::query_as()` instead of `sqlx::query_as!()`
- **Reason**: Compile-time verification requires database connection
- **Trade-off**: Still type-safe via FromRow derives, just verified at runtime

---

## 📦 Database Models (13 models)

### Location: `backend/db/src/models/`

#### **1. User Models** (`user.rs`)
```rust
User                 // User accounts (10 fields)
UserPreferences      // UI preferences (7 fields)
RefreshToken         // JWT tokens (8 fields)
```

#### **2. Company Model** (`company.rs`)
```rust
Company             // Company master data (17 fields)
```

#### **3. Financial Models** (`financials.rs`)
```rust
IncomeStatement     // Revenue, profit, EPS (22 fields)
BalanceSheet        // Assets, liabilities, equity (26 fields)
CashFlowStatement   // Cash flows (20 fields)
```

#### **4. Market Data** (`daily_price.rs`)
```rust
DailyPrice          // OHLCV data (11 fields)
```

#### **5. Analysis Models**
```rust
DerivedMetric       // Pre-computed metrics (derived_metric.rs)
Screener            // Stock screeners (screener.rs)
Verdict             // Investment verdicts (verdict.rs)
VerdictHistory      // Verdict history (verdict.rs)
```

#### **6. Document Models** (`document.rs`)
```rust
Document            // S3 document metadata (12 fields)
AnalysisReport      // User analysis files (6 fields)
```

### Model Features
- ✅ All use `#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]`
- ✅ `Uuid` for IDs
- ✅ `DateTime<Utc>` for timestamps
- ✅ `NaiveDate` for dates without time
- ✅ `BigDecimal` for financial amounts
- ✅ `Option<T>` for nullable fields
- ✅ `serde_json::Value` for JSONB fields

---

## 🗄️ Repository Layer (3 repositories, 38 methods)

### Location: `backend/db/src/repositories/`

### **1. User Repository** (`user.rs`)
**13 methods**

**User CRUD**:
- `find_by_username(username)` → `Option<User>`
- `find_by_email(email)` → `Option<User>`
- `find_by_id(id)` → `Option<User>`
- `create(CreateUserRequest)` → `User`
- `update_password(id, hash)` → `()`
- `update_last_login(id)` → `()`

**Refresh Tokens**:
- `create_refresh_token(user_id, hash, expires)` → `Uuid`
- `find_valid_refresh_token(hash)` → `Option<RefreshToken>`
- `revoke_refresh_token(id)` → `()`
- `revoke_all_user_tokens(user_id)` → `u64`
- `clean_expired_tokens()` → `u64`

**Preferences**:
- `get_preferences(user_id)` → `Option<UserPreferences>`
- `upsert_preferences(user_id, update)` → `UserPreferences`

**DTOs**:
- `CreateUserRequest` - For user creation
- `UserPreferencesUpdate` - For preference updates

### **2. Company Repository** (`company.rs`)
**16 methods**

**Company Queries**:
- `find_by_id(id)` → `Option<Company>`
- `find_by_symbol(symbol, exchange)` → `Option<Company>`
- `list(filters, pagination)` → `Vec<Company>` (dynamic WHERE clause)
- `search(query, limit)` → `Vec<Company>` (full-text search)

**Financial Queries**:
- `get_income_statements(company_id, period_type, limit)` → `Vec<IncomeStatement>`
- `get_balance_sheets(company_id, period_type, limit)` → `Vec<BalanceSheet>`
- `get_cash_flow_statements(company_id, period_type, limit)` → `Vec<CashFlowStatement>`
- `get_daily_prices(company_id, start, end)` → `Vec<DailyPrice>`

**Derived Metrics**:
- `get_derived_metrics(company_id, period_type, names)` → `Vec<DerivedMetric>`

**Upserts** (for background jobs):
- `upsert_income_statement(data)` → `IncomeStatement`
- `upsert_balance_sheet(data)` → `BalanceSheet`
- `upsert_cash_flow_statement(data)` → `CashFlowStatement`
- `upsert_daily_price(data)` → `DailyPrice`

**DTOs**:
- `Pagination` - Limit/offset (default: 50/0)
- `CompanyFilters` - Exchange, country, sector filters
- `IncomeStatementInsert` - 22 fields
- `BalanceSheetInsert` - 22 fields
- `CashFlowStatementInsert` - 16 fields
- `DailyPriceInsert` - 10 fields

**Key Features**:
- Dynamic filtering in `list()` method
- Full-text search with ts_rank ordering
- UPSERT with `ON CONFLICT DO UPDATE` pattern
- Consistent ordering (DESC for financials, ASC for prices)

### **3. Verdict Repository** (`verdict.rs`)
**9 methods**

**Queries**:
- `find_by_company(company_id, user_id)` → `Option<Verdict>`
- `find_by_id(verdict_id)` → `Option<Verdict>`

**Optimistic Locking**:
- `update_with_lock(id, update, expected_version)` → `Verdict`
  - Checks `lock_version == expected_version`
  - Increments lock_version on success
  - Returns `OptimisticLockError` on mismatch

**Insert**:
- `create(company_id, user_id, verdict)` → `Verdict`

**History**:
- `create_history_snapshot(verdict_id)` → `VerdictHistory`
- `get_history(verdict_id)` → `Vec<VerdictHistory>`
- `get_history_count(verdict_id)` → `i64`

**Delete**:
- `delete(verdict_id)` → `()`

**DTOs**:
- `VerdictCreate` - For creating verdicts
- `VerdictUpdate` - For updating verdicts (partial updates via COALESCE)

**Optimistic Locking Pattern**:
```rust
UPDATE verdicts
SET final_verdict = COALESCE($1, final_verdict),
    lock_version = lock_version + 1,  -- Auto-increment
    updated_at = $6
WHERE id = $7 AND lock_version = $8   -- Version check
RETURNING ...;
```

If 0 rows affected → fetch current version → return appropriate error

---

## 🔧 Error Handling

### DbError Enum (`error.rs`)
```rust
pub enum DbError {
    ConnectionError(String),
    QueryError(String),
    MigrationError(String),
    NotFound(String),
    DuplicateError(String),
    ValidationError(String),
    OptimisticLockError(String),  // ← NEW for verdict conflicts
    DatabaseError(String),
}
```

### Automatic SQLx Error Conversion
```rust
impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => DbError::NotFound(...),
            sqlx::Error::Database(db_err) => {
                if let Some(code) = db_err.code() {
                    if code == "23505" {  // Unique constraint
                        return DbError::DuplicateError(...);
                    }
                }
                DbError::DatabaseError(...)
            }
            _ => DbError::QueryError(...),
        }
    }
}
```

---

## 📋 Seed Data Summary

### Test User
- **Username**: `testuser`
- **Email**: `test@example.com`
- **Password**: `password123` (bcrypt hashed)
- **Purpose**: Development & authentication testing

### Sample Companies (5)
1. **AAPL** (Apple Inc.) - NASDAQ
   - Has 4 quarters of 2024 financial data
   - Income statements, balance sheets, cash flows
   - Derived metrics (margins, growth rates)
2. **MSFT** (Microsoft Corp.) - NASDAQ
3. **JPM** (JPMorgan Chase) - NYSE
4. **JNJ** (Johnson & Johnson) - NYSE
5. **TSLA** (Tesla Inc.) - NASDAQ

### Financial Data for AAPL
- **Q1 2024**: March 31, 2024
- **Q2 2024**: June 30, 2024
- **Q3 2024**: September 30, 2024
- **Q4 2024**: December 31, 2024

**Derived Metrics Included**:
- YoY Revenue Growth %
- Gross Margin %
- Operating Margin %
- Net Margin %
- FCF/Revenue %
- Return on Equity %

### Sample Screeners (2)
1. **High Growth Tech**
   - Exchange: NASDAQ
   - Sector: Technology
   - Market Cap: > $1B
   - Revenue Growth: > 20%

2. **Value Financials**
   - Sector: Financials
   - P/E: < 15
   - Dividend Yield: > 2%

---

## ⚙️ Build & Verification Status

### Compilation Status
```bash
$ cd backend && cargo check -p db
    Checking db v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.83s
```
✅ **All code compiles successfully**

### Known Warnings
- `sqlx-postgres v0.7.4` has future incompatibility warnings (from upstream)
- Not blocking, will be fixed in future SQLx versions

### File Permissions Issue
- `target/` directory has permission issues (likely from Docker)
- **Workaround**: Run from `backend/` directory, not project root
- **Command**: `cd backend && cargo check -p db`

---

## 🎯 Architectural Decisions

### 1. Database Type System
**Decision**: Use `BigDecimal` for all financial amounts  
**Alternatives Considered**: `rust_decimal`, `f64`  
**Rationale**:
- SQLx 0.7 has built-in `bigdecimal` support
- Precise decimal arithmetic (no floating-point errors)
- Compatible with PostgreSQL NUMERIC type
- Serde support for JSON serialization

### 2. Query Verification
**Decision**: Runtime verification (`query_as`) over compile-time (`query_as!`)  
**Rationale**:
- Compile-time requires database connection during build
- Runtime is still type-safe via `FromRow` derives
- Simpler development workflow
- Can add compile-time verification later with `sqlx prepare`

### 3. Optimistic Locking
**Decision**: Use `lock_version` column with atomic UPDATE  
**Rationale**:
- Prevents lost updates in concurrent scenarios
- Single query for check-and-update (atomic)
- Clear error messages on conflicts
- Industry-standard pattern

### 4. History Tracking
**Decision**: Explicit snapshots before updates  
**Alternatives**: Triggers, audit tables  
**Rationale**:
- Application controls what/when to snapshot
- Simpler than trigger-based
- Explicit in code (easier to understand)
- Can optimize (don't snapshot every minor change)

### 5. UPSERT Pattern
**Decision**: Use `ON CONFLICT DO UPDATE` for background jobs  
**Rationale**:
- Idempotent data imports
- Handles re-runs gracefully
- Single query (atomic)
- PostgreSQL native feature

---

## 📁 File Structure

```
backend/db/
├── Cargo.toml              ✅ Dependencies configured
├── .env                    ✅ DATABASE_URL for SQLx
├── README.md               ✅ Setup & usage docs
├── migrations/
│   ├── 001_initial_schema.sql   ✅ 1,389 lines
│   └── 002_seed_data.sql        ✅ 639 lines
└── src/
    ├── lib.rs              ✅ Pool init, migrations
    ├── error.rs            ✅ DbError enum
    ├── models/
    │   ├── mod.rs          ✅ Module declarations
    │   ├── user.rs         ✅ User, UserPreferences, RefreshToken
    │   ├── company.rs      ✅ Company
    │   ├── financials.rs   ✅ IncomeStatement, BalanceSheet, CashFlowStatement
    │   ├── daily_price.rs  ✅ DailyPrice
    │   ├── derived_metric.rs   ✅ DerivedMetric
    │   ├── screener.rs     ✅ Screener
    │   ├── verdict.rs      ✅ Verdict, VerdictHistory
    │   └── document.rs     ✅ Document, AnalysisReport
    └── repositories/
        ├── mod.rs          ✅ Module declarations & exports
        ├── user.rs         ✅ UserRepository (13 methods)
        ├── company.rs      ✅ CompanyRepository (16 methods)
        └── verdict.rs      ✅ VerdictRepository (9 methods)
```

**Total Stats**:
- 13 models
- 3 repositories
- 38 repository methods
- 146 model fields
- ~1,500 lines of Rust code

---

## 🚀 Next Steps (For Fresh Session)

### Phase 4.4: Git Checkpoint ⏳
```bash
cd /home/preetham/Documents/iap-alpha
git add backend/db/
git commit -m "feat(db): Complete database foundation with SQLx

- Add 25+ table schema with partitioning and full-text search
- Create seed data for 5 companies with financial statements
- Implement 13 database models with BigDecimal support
- Build 3 repositories with 38 methods
- Add optimistic locking for verdicts
- Configure SQLx with runtime query verification

Ref: build-plan-v3/04-database-foundation.md sections 4.1-4.3"

git push origin develop
```

### Phase 4.5: PostgreSQL Setup ⏳
**Choose ONE approach**:

**Option A: Docker (Recommended)**
```bash
docker run -d \
  --name irp-postgres \
  -e POSTGRES_PASSWORD=dev \
  -e POSTGRES_DB=irp_dev \
  -p 5432:5432 \
  -v irp_pgdata:/var/lib/postgresql/data \
  postgres:15-alpine
```

**Option B: Local PostgreSQL**
```bash
sudo apt install postgresql postgresql-contrib
sudo -u postgres psql
CREATE DATABASE irp_dev;
CREATE USER irp_user WITH PASSWORD 'dev';
GRANT ALL PRIVILEGES ON DATABASE irp_dev TO irp_user;
```

### Phase 4.6: Run Migrations ⏳
```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Set database URL
export DATABASE_URL="postgres://postgres:dev@localhost:5432/irp_dev"

# Run migrations
cd backend/db
sqlx migrate run

# Verify
psql $DATABASE_URL -c "\dt"  # List tables
psql $DATABASE_URL -c "SELECT COUNT(*) FROM companies;"  # Should show 5
```

### Phase 4.7: Verify with Test Queries ⏳
```bash
# Test user exists
psql $DATABASE_URL -c "SELECT username FROM users WHERE username = 'testuser';"

# Check Apple financial data
psql $DATABASE_URL -c "
    SELECT period_end_date, total_revenue, net_income 
    FROM income_statements 
    WHERE company_id = (SELECT id FROM companies WHERE symbol = 'AAPL')
    ORDER BY period_end_date DESC;
"

# Test full-text search
psql $DATABASE_URL -c "
    SELECT symbol, name 
    FROM companies 
    WHERE search_vector @@ plainto_tsquery('english', 'apple');
"
```

### Phase 4.8: Optional - SQLx Offline Mode ⏳
```bash
# Generate query metadata for compile-time verification
cd backend/db
cargo sqlx prepare

# This creates sqlx-data.json
# Allows compilation without database connection
```

---

## 💡 Important Notes for Resuming

### Environment Setup
1. **Working Directory**: Always run cargo commands from `backend/`, not project root
2. **Database URL**: Set in both `.env` file AND environment variable
3. **Permissions**: Target directory may need ownership fix if using Docker

### Testing Workflow
1. Start PostgreSQL (Docker or local)
2. Run migrations: `sqlx migrate run`
3. Verify with psql queries
4. Test repository methods (once API is built)

### Common Commands
```bash
# Check compilation
cd backend && cargo check -p db

# Run migrations
cd backend/db && sqlx migrate run

# Revert last migration
cd backend/db && sqlx migrate revert

# Connect to database
psql postgres://postgres:dev@localhost:5432/irp_dev

# View migration status
cd backend/db && sqlx migrate info
```

### Key Files to Reference
- Build Plan: `docs/build-plan-v3/04-database-foundation.md`
- Database Design: `docs/database-design-v1.md`
- Architecture: `docs/architecture-design-v3.md` (section 6.4 for optimistic locking)
- DB README: `backend/db/README.md`

---

## 📊 Completion Summary

| Phase | Status | Methods | Lines |
|-------|--------|---------|-------|
| 4.1 Schema | ✅ Complete | - | 1,389 |
| 4.2 Models | ✅ Complete | 13 models | ~500 |
| 4.3 Repositories | ✅ Complete | 38 methods | ~1,000 |
| 4.4 Git Checkpoint | ⏳ Pending | - | - |
| 4.5-4.7 Database Setup | ⏳ Pending | - | - |

**Overall Progress**: Database Foundation Phase ~75% Complete

The database layer is fully coded and compiles. Next session should focus on infrastructure (PostgreSQL setup) and running the actual migrations to populate the database!

---

**End of Walkthrough**  
**Ready to Resume**: Yes ✅  
**Next Build Plan Section**: Section 4.4-4.7 (Infrastructure & Verification)
