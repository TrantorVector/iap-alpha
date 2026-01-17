# Section 4: Database Foundation

**Time Required**: ~1-2 hours  
**Difficulty**: Medium  
**Goal**: Create PostgreSQL schema, migrations, and seed data

---

## Overview

This section sets up the complete database schema based on `database-design-v1.md`. We'll create:

1. **Migrations** - SQL files that create/modify tables
2. **Seed data** - Test data for development
3. **Repository layer** - Rust code for database access

---

## Database Schema Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      USER DATA                               │
├─────────────────────────────────────────────────────────────┤
│ users           │ User accounts, credentials                 │
│ user_preferences│ UI settings per user                       │
│ refresh_tokens  │ JWT refresh token storage                  │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    COMPANY DATA                              │
├─────────────────────────────────────────────────────────────┤
│ companies       │ Company master data                        │
│ exchanges       │ Stock exchanges (NASDAQ, NYSE, etc.)       │
│ sectors         │ Industry sectors                           │
│ currencies      │ Currency definitions (USD, INR)            │
│ fx_rates        │ Exchange rates                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   FINANCIAL DATA                             │
├─────────────────────────────────────────────────────────────┤
│ income_statements    │ Quarterly/annual income data          │
│ balance_sheets       │ Assets, liabilities, equity           │
│ cash_flow_statements │ Operating, investing, financing       │
│ daily_prices         │ Stock price history                   │
│ derived_metrics      │ Computed metrics (YoY, margins)       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    ANALYSIS DATA                             │
├─────────────────────────────────────────────────────────────┤
│ screeners       │ Saved filter configurations                │
│ verdicts        │ Current analysis verdicts                  │
│ verdict_history │ Historical verdict snapshots               │
│ documents       │ Document metadata (transcripts, etc.)      │
│ analysis_reports│ User-uploaded analysis PDFs                │
└─────────────────────────────────────────────────────────────┘
```

---

## Step-by-Step

### Step 4.1: Create Migration Files

---

#### 📋 PROMPT 4.1.1: Create Initial Schema Migration

```
Create SQLx migrations for the Investment Research Platform database.

Reference the database-design-v1.md document in docs/ folder for the complete schema.

Create migration file `backend/db/migrations/001_initial_schema.sql` with:

1. **Extension setup**:
   - Enable uuid-ossp extension for UUID generation

2. **Reference tables** (create first due to foreign keys):
   - currencies (code PK, name, symbol, decimal_places)
   - exchanges (code PK, name, country, timezone, currency FK, trading_days JSONB)
   - sectors (id UUID PK, name, parent_id self-reference)

3. **Users tables**:
   - users (id UUID PK, username UNIQUE, email UNIQUE, password_hash, created_at, updated_at)
   - user_preferences (id UUID PK, user_id FK UNIQUE, document_row_order JSONB, default_period_count, default_period_type, theme, updated_at)
   - refresh_tokens (id UUID PK, user_id FK, token_hash, expires_at, revoked, created_at)

4. **Companies tables**:
   - companies (id UUID PK, symbol, exchange FK, name, sector FK, market_cap, currency FK, fiscal_year_end_month, is_active, created_at, updated_at)
   - fx_rates (id UUID PK, from_currency FK, to_currency FK, rate, rate_date, created_at)

5. **Financial statements** (quarterly and annual data):
   - income_statements (id UUID PK, company_id FK, period_end_date, period_type, fiscal_year, fiscal_quarter, total_revenue, cost_of_revenue, gross_profit, operating_income, net_income, ebitda, basic_eps, diluted_eps, shares_outstanding, created_at)
   - balance_sheets (id UUID PK, company_id FK, period_end_date, period_type, total_assets, total_liabilities, total_equity, cash_and_equivalents, total_debt, net_debt, created_at)
   - cash_flow_statements (id UUID PK, company_id FK, period_end_date, period_type, operating_cash_flow, investing_cash_flow, financing_cash_flow, free_cash_flow, capital_expenditures, created_at)
   - daily_prices (id UUID PK, company_id FK, price_date, open, high, low, close, adjusted_close, volume, created_at)

6. **Derived metrics table**:
   - derived_metrics (id UUID PK, company_id FK, period_end_date, period_type, metric_name, metric_value, created_at)
   - Create indexes on (company_id, period_end_date, metric_name)

7. **Analysis tables**:
   - screeners (id UUID PK, user_id FK, title, description, filter_criteria JSONB, sort_config JSONB, display_columns JSONB, created_at, updated_at)
   - verdicts (id UUID PK, company_id FK UNIQUE, user_id FK, final_verdict, summary_text, strengths JSONB, weaknesses JSONB, guidance_summary, lock_version INT DEFAULT 0, created_at, updated_at)
   - verdict_history (id UUID PK, verdict_id FK, version INT, final_verdict, summary_text, recorded_at)

8. **Documents tables**:
   - documents (id UUID PK, company_id FK, document_type, period_end_date, title, storage_key, source_url, file_size, mime_type, created_at, updated_at)
   - analysis_reports (id UUID PK, verdict_id FK, verdict_history_id FK nullable, storage_key, filename, uploaded_at)

Create appropriate indexes for:
- Foreign keys
- Frequently queried columns (symbol, period_end_date)
- Unique constraints (company + period for financials)

Add UNIQUE constraints where appropriate to prevent duplicate data.
```

**Verification**: Migration file is syntactically correct SQL.

---

#### 📋 PROMPT 4.1.2: Create Seed Data Migration

```
Create a seed data migration for development and testing.

Create migration file `backend/db/migrations/002_seed_data.sql` with:

1. **Test user**:
   - Username: testuser
   - Email: test@example.com
   - Password hash: Use Argon2id hash for password "TestPass123!"
   - Note: Add a comment with the plain password for dev reference

2. **Reference data**:
   - Currencies: USD, INR
   - Exchanges: NASDAQ, NYSE, BSE (with trading_days JSON)
   - Sectors: Technology, Financial Services, Healthcare, Consumer Cyclical, Industrials

3. **Sample US companies** (for testing):
   - Apple (AAPL, NASDAQ)
   - Microsoft (MSFT, NASDAQ)
   - JPMorgan Chase (JPM, NYSE)
   - Johnson & Johnson (JNJ, NYSE)
   - Tesla (TSLA, NASDAQ)

4. **Sample financial data**:
   For Apple (AAPL), create 4 quarters of data:
   - Income statements (Q1-Q4 2024)
   - Balance sheets (Q1-Q4 2024)
   - Cash flow statements (Q1-Q4 2024)
   - Use realistic but approximate numbers

5. **Sample derived metrics**:
   For Apple, create computed metrics:
   - YoY Revenue Growth
   - Gross Margin %
   - Operating Margin %
   - Net Margin %
   - FCF/Revenue %

6. **Sample screener**:
   - Title: "High Growth Tech"
   - Filter criteria JSON: exchanges=NASDAQ, sector=Technology, market_cap > 100B

This data allows testing all application features without external API calls.
```

**Verification**: Seed data is inserted correctly.

---

### Step 4.2: Set Up SQLx in Backend

---

#### 📋 PROMPT 4.2.1: Configure SQLx for the Database Crate

```
Set up SQLx in the backend db crate for compile-time verified SQL queries.

1. Update `backend/db/Cargo.toml` with dependencies:
   - sqlx = { version = "0.7", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
   - uuid = { version = "1.0", features = ["v4", "serde"] }
   - chrono = { version = "0.4", features = ["serde"] }
   - serde = { version = "1.0", features = ["derive"] }
   - serde_json = "1.0"
   - async-trait = "0.1"
   - thiserror = "1.0"

2. Create `backend/db/src/lib.rs` with:
   - Module declarations for models and repositories
   - Database connection pool initialization function
   - Re-export of common types

3. Create database pool initialization:
   - Function: `pub async fn init_pool(database_url: &str) -> Result<PgPool>`
   - Set pool size (max 5 for dev)
   - Configure connection timeout
   - Test connection on initialization

4. Create `backend/db/.env` for SQLx offline mode:
   - DATABASE_URL=postgres://postgres:dev@localhost:5432/irp_dev

5. Add sqlx-cli to project:
   - Create instructions for installing: cargo install sqlx-cli
```

**Verification**: `cargo check -p db` compiles without errors.

---

#### 📋 PROMPT 4.2.2: Create Database Models

```
Create SQLx database models in `backend/db/src/models/`.

Create these model files:

1. `mod.rs` - Module declarations

2. `user.rs` - User and UserPreferences structs with:
   - UUID fields using sqlx::types::Uuid
   - DateTime fields using chrono::DateTime<Utc>
   - Serde derive for JSON serialization
   - sqlx::FromRow derive for database mapping

3. `company.rs` - Company struct with all fields from schema

4. `financials.rs` - IncomeStatement, BalanceSheet, CashFlowStatement structs

5. `daily_price.rs` - DailyPrice struct

6. `derived_metric.rs` - DerivedMetric struct

7. `screener.rs` - Screener struct with JSONB fields as serde_json::Value

8. `verdict.rs` - Verdict and VerdictHistory structs

9. `document.rs` - Document and AnalysisReport structs

Each model should:
- Derive: Debug, Clone, Serialize, Deserialize, sqlx::FromRow
- Use Option<T> for nullable fields
- Include all columns from the database schema
```

**Verification**: `cargo check -p db` compiles.

---

### Step 4.3: Create Repository Layer

---

#### 📋 PROMPT 4.3.1: Create User Repository

```
Create the user repository for authentication queries.

Create `backend/db/src/repositories/user_repository.rs` with:

1. Struct `UserRepository` holding a reference to `PgPool`

2. Methods:
   - `async fn find_by_username(&self, username: &str) -> Result<Option<User>>`
   - `async fn find_by_email(&self, email: &str) -> Result<Option<User>>`
   - `async fn find_by_id(&self, id: Uuid) -> Result<Option<User>>`
   - `async fn create(&self, user: CreateUserRequest) -> Result<User>`
   - `async fn update_password(&self, id: Uuid, password_hash: &str) -> Result<()>`

3. Methods for refresh tokens:
   - `async fn create_refresh_token(&self, user_id: Uuid, token_hash: &str, expires_at: DateTime<Utc>) -> Result<Uuid>`
   - `async fn find_valid_refresh_token(&self, token_hash: &str) -> Result<Option<RefreshToken>>`
   - `async fn revoke_refresh_token(&self, id: Uuid) -> Result<()>`
   - `async fn revoke_all_user_tokens(&self, user_id: Uuid) -> Result<u64>`

4. Methods for user preferences:
   - `async fn get_preferences(&self, user_id: Uuid) -> Result<Option<UserPreferences>>`
   - `async fn upsert_preferences(&self, user_id: Uuid, prefs: UserPreferencesUpdate) -> Result<UserPreferences>`

Use sqlx::query_as! macro for compile-time verified queries.
All queries should use parameterized inputs to prevent SQL injection.
```

**Verification**: `cargo check -p db` compiles.

---

#### 📋 PROMPT 4.3.2: Create Company Repository

```
Create the company repository for company and financial data queries.

Create `backend/db/src/repositories/company_repository.rs` with:

1. Struct `CompanyRepository` holding a reference to `PgPool`

2. Company query methods:
   - `async fn find_by_id(&self, id: Uuid) -> Result<Option<Company>>`
   - `async fn find_by_symbol(&self, symbol: &str, exchange: &str) -> Result<Option<Company>>`
   - `async fn list(&self, filters: CompanyFilters, pagination: Pagination) -> Result<Vec<Company>>`
   - `async fn search(&self, query: &str, limit: i32) -> Result<Vec<Company>>`

3. Financial data methods:
   - `async fn get_income_statements(&self, company_id: Uuid, period_type: &str, limit: i32) -> Result<Vec<IncomeStatement>>`
   - `async fn get_balance_sheets(&self, company_id: Uuid, period_type: &str, limit: i32) -> Result<Vec<BalanceSheet>>`
   - `async fn get_cash_flow_statements(&self, company_id: Uuid, period_type: &str, limit: i32) -> Result<Vec<CashFlowStatement>>`
   - `async fn get_daily_prices(&self, company_id: Uuid, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<DailyPrice>>`

4. Derived metrics methods:
   - `async fn get_derived_metrics(&self, company_id: Uuid, period_type: &str, metric_names: Vec<String>) -> Result<Vec<DerivedMetric>>`

5. Upsert methods for background job data insertion:
   - `async fn upsert_income_statement(&self, data: IncomeStatementInsert) -> Result<IncomeStatement>`
   - Similar for other financial data

Implement pagination with offset/limit pattern.
Include ORDER BY clauses for consistent results.
```

**Verification**: `cargo check -p db` compiles.

---

#### 📋 PROMPT 4.3.3: Create Verdict Repository

```
Create the verdict repository with optimistic locking support.

Create `backend/db/src/repositories/verdict_repository.rs` with:

1. Struct `VerdictRepository` holding a reference to `PgPool`

2. Query methods:
   - `async fn find_by_company(&self, company_id: Uuid) -> Result<Option<Verdict>>`
   - `async fn find_by_id(&self, verdict_id: Uuid) -> Result<Option<Verdict>>`

3. Optimistic locking update method:
   - `async fn update_with_lock(&self, verdict_id: Uuid, update: VerdictUpdate, expected_version: i32) -> Result<Verdict>`
   - Must check lock_version matches expected_version
   - Increment lock_version on success
   - Return custom error if version mismatch (OptimisticLockError)

4. Insert method:
   - `async fn create(&self, company_id: Uuid, user_id: Uuid, verdict: VerdictCreate) -> Result<Verdict>`

5. History methods:
   - `async fn create_history_snapshot(&self, verdict_id: Uuid) -> Result<VerdictHistory>`
   - `async fn get_history(&self, verdict_id: Uuid) -> Result<Vec<VerdictHistory>>`

The update_with_lock implementation should use a single UPDATE query with:
- WHERE verdict_id = $1 AND lock_version = $2
- Return row count to detect if update succeeded
- If 0 rows updated, fetch current version for error response

Reference architecture-design-v3.md section 6.4 for the optimistic locking pattern.
```

**Verification**: `cargo check -p db` compiles.

---

### Step 4.4: Run Migrations

Now let's apply the migrations to your Docker database.

**First, install sqlx-cli:**
```bash
cargo install sqlx-cli --features postgres
```

**Run migrations:**
```bash
# Make sure Docker is running
docker compose up -d

# Wait for postgres to be ready
sleep 5

# Create database (if needed)
sqlx database create --database-url postgres://postgres:dev@localhost:5432/irp_dev

# Run migrations
sqlx migrate run --source backend/db/migrations --database-url postgres://postgres:dev@localhost:5432/irp_dev
```

**Verify data:**
```bash
# Connect to database
docker exec -it iap-alpha-postgres-1 psql -U postgres -d irp_dev

# List tables
\dt

# Check seed data
SELECT * FROM users;
SELECT * FROM companies;

# Exit
\q
```

---

### Step 4.5: Git Checkpoint

```bash
git add .

git commit -m "feat(db): add database schema and repository layer

- Initial schema migration with all tables
- Seed data for development (test user, sample companies)
- SQLx models for all entities
- Repository pattern implementation:
  - UserRepository with auth methods
  - CompanyRepository with financial data access
  - VerdictRepository with optimistic locking
- Compile-time verified queries

Database follows database-design-v1.md specification."

git push origin develop
```

---

## Verification Checklist

After completing this section, verify:

- [ ] Migrations run without errors
- [ ] `\dt` in psql shows all expected tables
- [ ] Test user exists in database
- [ ] Sample companies exist in database
- [ ] `cargo check -p db` compiles without errors
- [ ] Commit pushed to GitHub

---

## Next Step

**Proceed to**: [05-backend-core.md](./05-backend-core.md)

---

## Troubleshooting

### Migration fails with "relation already exists"
```bash
# Reset database
sqlx database drop --database-url postgres://postgres:dev@localhost:5432/irp_dev
sqlx database create --database-url postgres://postgres:dev@localhost:5432/irp_dev
sqlx migrate run --source backend/db/migrations --database-url postgres://postgres:dev@localhost:5432/irp_dev
```

### SQLx compile errors about missing DATABASE_URL
```bash
# Create .env file
echo "DATABASE_URL=postgres://postgres:dev@localhost:5432/irp_dev" > backend/db/.env

# Or use offline mode
cargo sqlx prepare --database-url postgres://postgres:dev@localhost:5432/irp_dev
```

### Can't connect to database
```bash
# Check postgres is running
docker compose ps

# Check logs
docker compose logs postgres
```
