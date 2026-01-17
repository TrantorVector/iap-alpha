# Section 10: Background Jobs

**Time Required**: ~2 hours  
**Difficulty**: Medium  
**Goal**: Build the worker binary with nightly background jobs (Priority 3)

---

## Overview

Background jobs handle:
- **Nightly data refresh**: Earnings calendar, prices, FX rates
- **Document fetching**: Download transcripts, filings
- **Metrics recalculation**: Update derived metrics

Reference: Architecture sections 9.1-9.4

---

## Job Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    EventBridge                           │
│              (Cron: Daily at 2 AM IST)                   │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    ECS Fargate Task                      │
│                    (Worker Binary)                       │
├─────────────────────────────────────────────────────────┤
│ Jobs:                                                   │
│ - EarningsCalendarPollingJob                            │
│ - PriceRefreshJob                                       │
│ - FxRateRefreshJob                                      │
│ - DocumentRefreshJob                                    │
│ - MetricsRecalculationJob                               │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
              ┌────────────┴────────────┐
              │                         │
              ▼                         ▼
         PostgreSQL              External APIs
                              (Mock for development)
```

---

## Step-by-Step

### Step 10.1: Mock Provider Implementation

We'll use mock providers for development to avoid API rate limits.

---

#### 📋 PROMPT 10.1.1: Create Mock Market Data Provider

```
Create a mock market data provider using golden-copy data.

Create `backend/providers/src/mock/mod.rs`:

1. `MockMarketDataProvider` struct implementing `MarketDataProvider` trait

2. Load responses from files or embedded data:
   - Use the golden-copy folder data structure
   - Parse JSON responses into domain types

3. Implement all trait methods:
   - get_company_overview: Return mock Apple/Microsoft data
   - get_income_statement: Return quarterly data from files
   - get_balance_sheet: Return from files
   - get_cash_flow: Return from files
   - get_daily_prices: Return sample price history
   - get_earnings_calendar: Return upcoming earnings

4. Add configurable delays to simulate API latency

5. Allow configuration via environment:
   - MOCK_DATA_PATH: Path to golden-copy folder
   - MOCK_API_DELAY_MS: Simulated latency

The mock provider should be the default for development.
Use real providers only when ENVIRONMENT=production.
```

**Verification**: Mock provider returns data.

---

### Step 10.2: Worker Binary Structure

---

#### 📋 PROMPT 10.2.1: Create Worker Binary

```
Create the background worker binary.

Create `backend/worker/src/main.rs`:

1. Parse command line arguments:
   - --job: Specific job to run (optional)
   - --all: Run all nightly jobs in sequence

2. Job definitions:
   - earnings_poll
   - price_refresh
   - fx_refresh
   - document_refresh
   - metrics_recalc

3. Main logic:
   - Initialize configuration
   - Initialize database pool
   - Initialize providers (mock or real based on env)
   - Run specified job(s)
   - Log results and exit

4. Error handling:
   - Log job failures
   - Continue with next job on failure
   - Exit with non-zero code if any job fails

Create `backend/worker/src/jobs/mod.rs` with job trait and implementations.
```

**Verification**: Worker compiles and runs with `--help`.

---

### Step 10.3: Individual Jobs

---

#### 📋 PROMPT 10.3.1: Create Earnings Calendar Job

```
Create the earnings calendar polling job.

Create `backend/worker/src/jobs/earnings_poll.rs`:

1. `EarningsPollingJob` struct with:
   - db: PgPool
   - provider: Arc<dyn MarketDataProvider>

2. `run()` method:
   a. Create job_run record in database
   b. Get list of active tracked companies
   c. For each company:
      - Fetch earnings calendar from provider
      - Check for new/updated earnings dates
      - Update company if earnings date changed
      - Log updates
   d. Update job_run record with results
   e. Return JobResult { processed, updated, errors }

3. Handle rate limiting:
   - Respect provider rate limits
   - Use tiered provider if primary exhausted

4. Logging:
   - Use tracing with structured fields
   - Log company_id, old_date, new_date for changes
```

**Verification**: Job runs and logs results.

---

#### 📋 PROMPT 10.3.2: Create Price Refresh Job

```
Create the daily price refresh job.

Create `backend/worker/src/jobs/price_refresh.rs`:

1. `PriceRefreshJob` struct

2. `run()` method:
   a. Get companies needing price update:
      - Active companies
      - Last price older than 1 trading day
   b. For each company:
      - Fetch latest daily prices
      - Upsert into daily_prices table
      - Update market_cap from latest close * shares
   c. Log summary

3. Market cap calculation:
   - latest_close * shares_outstanding
   - Handle missing data gracefully

4. Skip weekends/holidays (trading day check)
```

**Verification**: Job updates price data.

---

#### 📋 PROMPT 10.3.3: Create Metrics Recalculation Job

```
Create the metrics recalculation job.

Create `backend/worker/src/jobs/metrics_recalc.rs`:

1. `MetricsRecalculationJob` struct

2. `run()` method:
   a. Get companies with updated financials (since last recalc)
   b. For each company:
      - Load income statements, balance sheets, cash flows
      - Calculate all derived metrics
      - Upsert into derived_metrics table
   c. Log summary

3. Metrics to calculate per PRD FR-SCR-004:
   - YoY Revenue Growth
   - QoQ Revenue Growth
   - Growth Acceleration
   - Gross Margin %
   - Operating Margin %
   - Net Margin %
   - OCF/Revenue %
   - FCF/Revenue %
   - (Revenue - Net Debt) / Revenue %
   - P/E Ratio (from latest price)
   - Momentum 1M, 3M, 6M (from prices)

4. Store with company_id, period_end_date, metric_name, value
```

**Verification**: Derived metrics are populated.

---

### Step 10.4: Job Scheduler (Local)

---

#### 📋 PROMPT 10.4.1: Create Local Job Runner

```
Create a local development job runner with scheduling.

Create `backend/worker/src/scheduler.rs`:

1. Development mode scheduling:
   - Run jobs immediately on --run-now flag
   - Or run on a simple cron schedule

2. Create Docker service for worker:
   Update docker-compose.yml:
   ```yaml
   worker:
     build:
       context: ./backend
       dockerfile: Dockerfile.dev
     environment:
       - DATABASE_URL=postgres://postgres:dev@postgres:5432/irp_dev
       - RUST_LOG=debug
     depends_on:
       - postgres
     command: cargo run -p worker -- --all
     profiles:
       - jobs  # Only run when explicitly requested
   ```

3. Usage:
   - `docker compose --profile jobs up worker` - Run once
   - `docker compose exec api cargo run -p worker -- --job metrics_recalc`

This replaces EventBridge for local development.
```

**Verification**: Can trigger jobs via docker compose.

---

### Step 10.5: Integration Tests for Jobs

---

#### 📋 PROMPT 10.5.1: Create Job Integration Tests

```
Create integration tests for background jobs.

Create `tests/integration/jobs_test.rs`:

1. Setup:
   - Create test database
   - Seed with sample companies
   - Use mock providers

2. Tests:
   - `test_earnings_poll_updates_calendar`
   - `test_price_refresh_updates_prices`
   - `test_price_refresh_updates_market_cap`
   - `test_metrics_recalc_creates_derived_metrics`
   - `test_job_records_success_in_database`
   - `test_job_handles_provider_errors_gracefully`

3. Each test should:
   - Run the job
   - Verify database was updated
   - Check job_runs table has record
```

**Verification**: Job tests pass.

---

### Step 10.6: Git Checkpoint

```bash
# Run worker once to test
docker compose exec api cargo run -p worker -- --job metrics_recalc

# Check derived_metrics table
docker exec -it iap-alpha-postgres-1 psql -U postgres -d irp_dev -c "SELECT * FROM derived_metrics LIMIT 10;"

# Commit
git add .

git commit -m "feat(worker): implement background job system

Jobs:
- Earnings calendar polling
- Price refresh with market cap updates
- Metrics recalculation for all derived metrics

Infrastructure:
- Mock provider using golden-copy data
- Job runner binary with CLI
- Local scheduler via Docker Compose profile

Development:
- Jobs use mock data to avoid API limits
- Run manually or on schedule
- Integration tests for all jobs"

git push origin develop
```

---

## Verification Checklist

- [ ] Worker binary compiles and runs
- [ ] Mock provider returns realistic data
- [ ] Each job executes without errors
- [ ] Database is updated after job runs
- [ ] job_runs table tracks execution
- [ ] Integration tests pass
- [ ] Commit pushed to GitHub

---

## Running Jobs

```bash
# Run all jobs once
docker compose exec api cargo run -p worker -- --all

# Run specific job
docker compose exec api cargo run -p worker -- --job metrics_recalc

# Run with the jobs profile (dedicated container)
docker compose --profile jobs up worker
```

---

## Next Step

**Proceed to**: [11-results-tracker.md](./11-results-tracker.md)
