# Investment Research Platform - Task List

**Last Updated**: January 17, 2026, 4:56 PM IST  
**Current Phase**: Database Foundation (Step 4)  
**Status**: Phases 4.1-4.3 Complete, 4.4-4.7 Pending

---

## ✅ Completed Tasks

### Planning & Design Phase
- [x] Product Requirements Document (PRD v1.4)
- [x] Database Design Document (v1)
- [x] Architecture Design Document (v3)
- [x] Build Plan (v3 - 7 phases)
- [x] Repository initialization with Git
- [x] Directory structure created
- [x] Docker Compose configuration

### Database Foundation (Step 4.1-4.3)
- [x] **4.1.1** Create initial database schema (25+ tables, 1,389 lines)
  - Users & authentication tables
  - Company & reference data tables
  - Financial statement tables (partitioned)
  - Derived metrics & analysis tables
  - Document management tables
  - System tables (API cache, background jobs)
  
- [x] **4.1.2** Create seed data migration (5 companies, 639 lines)
  - Test user with bcrypt password
  - 5 sample companies (AAPL, MSFT, JPM, JNJ, TSLA)
  - 4 quarters of 2024 financial data for Apple
  - Derived metrics (margins, growth rates)
  - 2 sample screeners

- [x] **4.2.1** Configure SQLx crate
  - Added dependencies (SQLx, BigDecimal, UUID, Chrono)
  - Created .env file with DATABASE_URL
  - Set up lib.rs with pool initialization
  - Created error module with DbError enum

- [x] **4.2.2** Create database models (13 models, 146 fields)
  - User models (User, UserPreferences, RefreshToken)
  - Company model
  - Financial models (IncomeStatement, BalanceSheet, CashFlowStatement)
  - Market data models (DailyPrice, DerivedMetric)
  - Analysis models (Screener, Verdict, VerdictHistory)
  - Document models (Document, AnalysisReport)

- [x] **4.3.1** Create User Repository (13 methods)
  - User CRUD operations (6 methods)
  - Refresh token management (5 methods)
  - User preferences (2 methods)
  - DTOs: CreateUserRequest, UserPreferencesUpdate

- [x] **4.3.2** Create Company Repository (16 methods)
  - Company queries (4 methods)
  - Financial statement queries (4 methods)
  - Derived metrics queries (1 method)
  - Upsert methods for background jobs (4 methods)
  - DTOs: Pagination, CompanyFilters, 4 Insert DTOs

- [x] **4.3.3** Create Verdict Repository (9 methods)
  - Query methods (2 methods)
  - Optimistic locking update (1 method)
  - Insert method (1 method)
  - History management (3 methods)
  - Delete method (1 method)
  - DTOs: VerdictCreate, VerdictUpdate

---

## 🔄 In Progress

### Database Foundation (Step 4.4-4.7)
Currently working on infrastructure and verification phases.

---

## ⏳ Pending Tasks

### Database Foundation - Infrastructure

#### **4.4 Git Checkpoint & Documentation** ⬅️ NEXT
- [ ] Add all database code to Git
- [ ] Commit with descriptive message
- [ ] Push to develop branch
- [ ] Update CHANGELOG.md

#### **4.5 PostgreSQL Setup**
**Choose ONE approach:**

##### Option A: Docker (Recommended)
- [ ] Pull PostgreSQL 15 Alpine image
- [ ] Create Docker volume for persistence
- [ ] Run PostgreSQL container with:
  - Database: `irp_dev`
  - User: `postgres`
  - Password: `dev`
  - Port: 5432
- [ ] Verify container is running
- [ ] Test connection with psql

##### Option B: Local Installation
- [ ] Install PostgreSQL 15+
- [ ] Create `irp_dev` database
- [ ] Create user with appropriate permissions
- [ ] Configure authentication (pg_hba.conf)
- [ ] Test connection

#### **4.6 Run Database Migrations**
- [ ] Install sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`
- [ ] Set DATABASE_URL environment variable
- [ ] Navigate to `backend/db/`
- [ ] Run `sqlx migrate run`
- [ ] Verify migration status with `sqlx migrate info`
- [ ] Check for errors in migration output

#### **4.7 Verify Database with Test Queries**
- [ ] Connect to database with psql
- [ ] List all tables (`\dt`)
- [ ] Count companies: `SELECT COUNT(*) FROM companies;` (expect 5)
- [ ] Verify test user exists
- [ ] Check Apple financial data (4 quarters)
- [ ] Test full-text search on companies
- [ ] Verify partitions exist for daily_prices
- [ ] Test computed columns (total_debt, net_debt, free_cash_flow)
- [ ] Verify triggers (updated_at auto-update)

#### **4.8 Optional: SQLx Offline Mode**
- [ ] Run `cargo sqlx prepare` to generate query metadata
- [ ] Verify `sqlx-data.json` created
- [ ] Test compilation without database connection
- [ ] Commit sqlx-data.json to Git

---

### Backend API Development (Step 5) - FUTURE

#### **5.1 Backend Project Setup**
- [ ] Create Axum API project structure
- [ ] Configure workspace Cargo.toml
- [ ] Set up module structure (routes, handlers, middleware)
- [ ] Configure environment variables
- [ ] Set up logging (tracing/tracing-subscriber)

#### **5.2 Authentication & JWT**
- [ ] Implement JWT token generation
- [ ] Implement JWT token validation
- [ ] Create auth middleware
- [ ] Implement login endpoint
- [ ] Implement logout endpoint
- [ ] Implement refresh token endpoint
- [ ] Implement password hashing (bcrypt)

#### **5.3 Core API Endpoints**
**User Management:**
- [ ] POST /api/auth/register
- [ ] POST /api/auth/login
- [ ] POST /api/auth/logout
- [ ] POST /api/auth/refresh
- [ ] GET /api/users/me
- [ ] PATCH /api/users/preferences

**Companies:**
- [ ] GET /api/companies (list with filters)
- [ ] GET /api/companies/:id
- [ ] GET /api/companies/search?q=

**Financial Data:**
- [ ] GET /api/companies/:id/financials/income
- [ ] GET /api/companies/:id/financials/balance-sheet
- [ ] GET /api/companies/:id/financials/cash-flow
- [ ] GET /api/companies/:id/prices?start=&end=
- [ ] GET /api/companies/:id/metrics

**Screeners:**
- [ ] GET /api/screeners
- [ ] POST /api/screeners
- [ ] PUT /api/screeners/:id
- [ ] DELETE /api/screeners/:id
- [ ] POST /api/screeners/:id/execute

**Verdicts:**
- [ ] GET /api/verdicts/:companyId
- [ ] POST /api/verdicts
- [ ] PUT /api/verdicts/:id (with optimistic locking)
- [ ] GET /api/verdicts/:id/history

#### **5.4 Error Handling & Validation**
- [ ] Create custom error types
- [ ] Implement error response middleware
- [ ] Add request validation (validator crate)
- [ ] Handle database errors gracefully
- [ ] Handle authentication errors
- [ ] Handle optimistic lock errors

#### **5.5 Testing**
- [ ] Set up integration test infrastructure
- [ ] Test authentication endpoints
- [ ] Test CRUD operations
- [ ] Test optimistic locking scenario
- [ ] Test error cases
- [ ] Set up test database fixtures

---

### Frontend Development (Step 6) - FUTURE

#### **6.1 Frontend Project Setup**
- [ ] Create React + TypeScript project (Vite)
- [ ] Set up TailwindCSS
- [ ] Configure routing (React Router)
- [ ] Set up state management (Context/Redux)
- [ ] Configure API client (Axios/Fetch)

#### **6.2 Authentication UI**
- [ ] Login page
- [ ] Registration page
- [ ] JWT token storage & refresh
- [ ] Protected route wrapper
- [ ] Logout functionality

#### **6.3 Core Pages**
- [ ] Dashboard/Home page
- [ ] Company search/list page
- [ ] Company detail page
- [ ] Financial statement viewer
- [ ] Screener builder interface
- [ ] Verdict editor with optimistic locking handling

#### **6.4 Components**
- [ ] Navigation/Header
- [ ] Financial table components
- [ ] Chart components (price charts)
- [ ] Form components
- [ ] Loading states
- [ ] Error boundaries

---

### Background Jobs & Data Pipeline (Step 7) - FUTURE

#### **7.1 Background Job System**
- [ ] Set up job queue (using background_jobs table)
- [ ] Create job processor
- [ ] Implement retry logic
- [ ] Add job monitoring

#### **7.2 Alpha Vantage Integration**
- [ ] Implement Alpha Vantage API client
- [ ] Add rate limiting (5 calls/minute)
- [ ] Implement API caching
- [ ] Create data transformation layer

#### **7.3 Data Fetching Jobs**
- [ ] Fetch company overview job
- [ ] Fetch income statement job
- [ ] Fetch balance sheet job
- [ ] Fetch cash flow job
- [ ] Fetch daily prices job
- [ ] Calculate derived metrics job

---

## 🎯 Current Focus

**Immediate Next Steps:**
1. ✅ Complete database code (DONE)
2. ✅ Verify compilation (DONE)
3. ⬅️ **NOW**: Git checkpoint (commit & push)
4. **NEXT**: Set up PostgreSQL (Docker recommended)
5. **THEN**: Run migrations
6. **FINALLY**: Verify with test queries

**Today's Goal**: Complete Step 4 (Database Foundation) entirely

---

## 📊 Progress Overview

| Phase | Status | Progress |
|-------|--------|----------|
| 1. Planning & Design | ✅ Complete | 100% |
| 2. Repository Setup | ✅ Complete | 100% |
| 3. Docker Setup | ✅ Complete | 100% |
| 4. Database Foundation | 🔄 In Progress | 75% |
| 5. Backend API | ⏳ Pending | 0% |
| 6. Frontend | ⏳ Pending | 0% |
| 7. Data Pipeline | ⏳ Pending | 0% |

**Overall Project Progress**: ~25%

---

## 📝 Notes & Decisions

### Key Architectural Decisions
1. **BigDecimal for Financials**: Using `bigdecimal` v0.3 for precise decimal arithmetic
2. **Runtime Query Verification**: Using `query_as()` instead of `query_as!()` for simpler dev workflow
3. **Optimistic Locking**: Implemented for verdicts to handle concurrent updates
4. **Partitioning**: Daily prices partitioned by year for performance
5. **Full-Text Search**: Implemented with tsvector for company search

### Important URLs & Commands
```bash
# Project root
cd /home/preetham/Documents/iap-alpha

# Compile database crate
cd backend && cargo check -p db

# Connect to database
psql postgres://postgres:dev@localhost:5432/irp_dev

# Run migrations
cd backend/db && sqlx migrate run
```

### Known Issues
- `target/` directory permission issues (run from `backend/`, not root)
- SQLx postgres v0.7.4 has future incompatibility warnings (upstream issue)

---

## 🔗 Reference Documentation

- Build Plan: `docs/build-plan-v3/04-database-foundation.md`
- Database Design: `docs/database-design-v1.md`
- Architecture: `docs/architecture-design-v3.md`
- Session Walkthrough: `.gemini/artifacts/database_foundation_walkthrough.md`
- DB README: `backend/db/README.md`

---

**End of Task List**  
**Last Review**: January 17, 2026  
**Next Review**: After completing Step 4
