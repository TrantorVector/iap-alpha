-- Migration: 001_initial_schema.sql
-- Description: Initial database schema for Investment Research Platform
-- Date: 2026-01-17
-- Reference: docs/database-design-v1.md

-- ============================================================================
-- EXTENSIONS
-- ============================================================================

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable trigram search for fuzzy text matching
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- ============================================================================
-- REFERENCE TABLES
-- ============================================================================

-- Currencies table
CREATE TABLE currencies (
    code                VARCHAR(10) PRIMARY KEY,
    name                VARCHAR(100) NOT NULL,
    symbol              VARCHAR(10) NOT NULL,
    decimal_places      INTEGER NOT NULL DEFAULT 2,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert common currencies
INSERT INTO currencies (code, name, symbol, decimal_places) VALUES
    ('USD', 'US Dollar', '$', 2),
    ('INR', 'Indian Rupee', '₹', 2),
    ('EUR', 'Euro', '€', 2),
    ('GBP', 'British Pound', '£', 2),
    ('JPY', 'Japanese Yen', '¥', 0);

-- Exchanges table
CREATE TABLE exchanges (
    code                VARCHAR(20) PRIMARY KEY,
    name                VARCHAR(255) NOT NULL,
    country             VARCHAR(50) NOT NULL,
    timezone            VARCHAR(50) NOT NULL,
    currency            VARCHAR(10) NOT NULL REFERENCES currencies(code),
    trading_days        JSONB,  -- e.g., {"days": ["Monday", "Tuesday", ...], "holidays": [...]}
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert major exchanges
INSERT INTO exchanges (code, name, country, timezone, currency) VALUES
    ('NASDAQ', 'NASDAQ Stock Market', 'USA', 'America/New_York', 'USD'),
    ('NYSE', 'New York Stock Exchange', 'USA', 'America/New_York', 'USD'),
    ('NSE', 'National Stock Exchange of India', 'India', 'Asia/Kolkata', 'INR'),
    ('BSE', 'Bombay Stock Exchange', 'India', 'Asia/Kolkata', 'INR');

-- Sectors table (hierarchical)
CREATE TABLE sectors (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name                VARCHAR(100) NOT NULL UNIQUE,
    parent_id           UUID REFERENCES sectors(id) ON DELETE SET NULL,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert common sectors
INSERT INTO sectors (name) VALUES
    ('Technology'),
    ('Healthcare'),
    ('Financials'),
    ('Consumer Cyclical'),
    ('Consumer Defensive'),
    ('Industrials'),
    ('Energy'),
    ('Utilities'),
    ('Real Estate'),
    ('Basic Materials'),
    ('Communication Services');

-- ============================================================================
-- USER TABLES
-- ============================================================================

-- Users table
CREATE TABLE users (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username            VARCHAR(50) NOT NULL UNIQUE,
    email               VARCHAR(255) NOT NULL UNIQUE,
    password_hash       VARCHAR(255) NOT NULL,
    
    -- Profile
    display_name        VARCHAR(100),
    timezone            VARCHAR(50) DEFAULT 'Asia/Kolkata',
    
    -- Status
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at       TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT chk_username_length CHECK (char_length(username) >= 3),
    CONSTRAINT chk_email_format CHECK (email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$')
);

-- Create index for login queries
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);

-- User preferences table
CREATE TABLE user_preferences (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id                 UUID NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
    
    -- UI Preferences
    document_row_order      JSONB DEFAULT '["investor_presentation", "earnings_call_transcript", "earnings_release"]',
    default_period_count    INTEGER DEFAULT 4,
    default_period_type     VARCHAR(20) DEFAULT 'quarterly' CHECK (default_period_type IN ('quarterly', 'annual')),
    theme                   VARCHAR(20) DEFAULT 'light' CHECK (theme IN ('light', 'dark', 'auto')),
    
    -- Audit
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Refresh tokens for JWT authentication
CREATE TABLE refresh_tokens (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash          VARCHAR(255) NOT NULL,
    
    -- Token metadata
    expires_at          TIMESTAMPTZ NOT NULL,
    revoked             BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Device tracking
    device_info         JSONB,
    ip_address          INET,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_token_hash UNIQUE (token_hash)
);

-- Indexes for refresh tokens
CREATE INDEX idx_refresh_tokens_user ON refresh_tokens(user_id);
CREATE INDEX idx_refresh_tokens_expires ON refresh_tokens(expires_at);
CREATE INDEX idx_refresh_tokens_revoked ON refresh_tokens(revoked, expires_at);

-- ============================================================================
-- COMPANY TABLES
-- ============================================================================

-- Companies master table
CREATE TABLE companies (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    symbol              VARCHAR(20) NOT NULL,
    exchange            VARCHAR(20) NOT NULL REFERENCES exchanges(code),
    name                VARCHAR(255) NOT NULL,
    
    -- Classification
    sector_id           UUID REFERENCES sectors(id),
    industry            VARCHAR(100),
    country             VARCHAR(50),
    
    -- Financial metadata
    market_cap          BIGINT,
    currency            VARCHAR(10) REFERENCES currencies(code),
    fiscal_year_end_month INTEGER CHECK (fiscal_year_end_month BETWEEN 1 AND 12),
    
    -- Additional info
    description         TEXT,
    cik                 VARCHAR(20),  -- SEC CIK for US companies
    address             TEXT,
    latest_quarter      DATE,
    
    -- Status
    is_active           BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Full-text search
    search_vector       TSVECTOR GENERATED ALWAYS AS (
                            setweight(to_tsvector('english', coalesce(name, '')), 'A') ||
                            setweight(to_tsvector('english', coalesce(symbol, '')), 'A') ||
                            setweight(to_tsvector('english', coalesce(description, '')), 'B')
                        ) STORED,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_symbol_exchange UNIQUE (symbol, exchange)
);

-- Indexes for companies
CREATE INDEX idx_companies_search ON companies USING GIN (search_vector);
CREATE INDEX idx_companies_exchange ON companies(exchange);
CREATE INDEX idx_companies_sector ON companies(sector_id);
CREATE INDEX idx_companies_country ON companies(country);
CREATE INDEX idx_companies_market_cap ON companies(market_cap);
CREATE INDEX idx_companies_is_active ON companies(is_active);

-- FX rates table
CREATE TABLE fx_rates (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_currency       VARCHAR(10) NOT NULL REFERENCES currencies(code),
    to_currency         VARCHAR(10) NOT NULL REFERENCES currencies(code),
    rate                DECIMAL(15, 6) NOT NULL,
    rate_date           DATE NOT NULL,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_fx_pair_date UNIQUE (from_currency, to_currency, rate_date),
    CONSTRAINT chk_different_currencies CHECK (from_currency != to_currency)
);

-- Indexes for FX rates
CREATE INDEX idx_fx_rates_pair_date ON fx_rates(from_currency, to_currency, rate_date DESC);

-- ============================================================================
-- FINANCIAL STATEMENT TABLES
-- ============================================================================

-- Income statements
CREATE TABLE income_statements (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id              UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    period_end_date         DATE NOT NULL,
    period_type             VARCHAR(10) NOT NULL CHECK (period_type IN ('quarterly', 'annual')),
    fiscal_year             INTEGER,
    fiscal_quarter          INTEGER CHECK (fiscal_quarter BETWEEN 1 AND 4),
    
    -- Revenue & Profit
    total_revenue           DECIMAL(20, 2),
    cost_of_revenue         DECIMAL(20, 2),
    gross_profit            DECIMAL(20, 2),
    
    -- Operating
    operating_expenses      DECIMAL(20, 2),
    operating_income        DECIMAL(20, 2),
    
    -- Net income
    interest_income         DECIMAL(20, 2),
    interest_expense        DECIMAL(20, 2),
    income_before_tax       DECIMAL(20, 2),
    income_tax_expense      DECIMAL(20, 2),
    net_income              DECIMAL(20, 2),
    
    -- Non-cash items
    depreciation_amortization DECIMAL(20, 2),
    ebit                    DECIMAL(20, 2),
    ebitda                  DECIMAL(20, 2),
    
    -- Per share metrics
    basic_eps               DECIMAL(10, 4),
    diluted_eps             DECIMAL(10, 4),
    shares_outstanding      BIGINT,
    
    -- Audit
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_income_company_period UNIQUE (company_id, period_end_date, period_type)
);

-- Indexes for income statements
CREATE INDEX idx_income_company ON income_statements(company_id);
CREATE INDEX idx_income_period_date ON income_statements(period_end_date DESC);
CREATE INDEX idx_income_company_period ON income_statements(company_id, period_end_date DESC);

-- Balance sheets
CREATE TABLE balance_sheets (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id              UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    period_end_date         DATE NOT NULL,
    period_type             VARCHAR(10) NOT NULL CHECK (period_type IN ('quarterly', 'annual')),
    fiscal_year             INTEGER,
    fiscal_quarter          INTEGER CHECK (fiscal_quarter BETWEEN 1 AND 4),
    
    -- Assets
    total_assets            DECIMAL(20, 2),
    current_assets          DECIMAL(20, 2),
    cash_and_equivalents    DECIMAL(20, 2),
    short_term_investments  DECIMAL(20, 2),
    inventory               DECIMAL(20, 2),
    accounts_receivable     DECIMAL(20, 2),
    non_current_assets      DECIMAL(20, 2),
    property_plant_equipment DECIMAL(20, 2),
    goodwill                DECIMAL(20, 2),
    intangible_assets       DECIMAL(20, 2),
    
    -- Liabilities
    total_liabilities       DECIMAL(20, 2),
    current_liabilities     DECIMAL(20, 2),
    accounts_payable        DECIMAL(20, 2),
    short_term_debt         DECIMAL(20, 2),
    non_current_liabilities DECIMAL(20, 2),
    long_term_debt          DECIMAL(20, 2),
    
    -- Equity
    total_equity            DECIMAL(20, 2),
    retained_earnings       DECIMAL(20, 2),
    common_stock            DECIMAL(20, 2),
    
    -- Derived
    total_debt              DECIMAL(20, 2) GENERATED ALWAYS AS (
                                COALESCE(short_term_debt, 0) + COALESCE(long_term_debt, 0)
                            ) STORED,
    net_debt                DECIMAL(20, 2) GENERATED ALWAYS AS (
                                COALESCE(short_term_debt, 0) + COALESCE(long_term_debt, 0) - COALESCE(cash_and_equivalents, 0)
                            ) STORED,
    
    -- Audit
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_balance_company_period UNIQUE (company_id, period_end_date, period_type)
);

-- Indexes for balance sheets
CREATE INDEX idx_balance_company ON balance_sheets(company_id);
CREATE INDEX idx_balance_period_date ON balance_sheets(period_end_date DESC);
CREATE INDEX idx_balance_company_period ON balance_sheets(company_id, period_end_date DESC);

-- Cash flow statements
CREATE TABLE cash_flow_statements (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id              UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    period_end_date         DATE NOT NULL,
    period_type             VARCHAR(10) NOT NULL CHECK (period_type IN ('quarterly', 'annual')),
    fiscal_year             INTEGER,
    fiscal_quarter          INTEGER CHECK (fiscal_quarter BETWEEN 1 AND 4),
    
    -- Operating activities
    operating_cash_flow     DECIMAL(20, 2),
    net_income              DECIMAL(20, 2),
    depreciation_depletion  DECIMAL(20, 2),
    change_in_receivables   DECIMAL(20, 2),
    change_in_inventory     DECIMAL(20, 2),
    change_in_payables      DECIMAL(20, 2),
    
    -- Investing activities
    investing_cash_flow     DECIMAL(20, 2),
    capital_expenditures    DECIMAL(20, 2),
    investments             DECIMAL(20, 2),
    
    -- Financing activities
    financing_cash_flow     DECIMAL(20, 2),
    dividend_payout         DECIMAL(20, 2),
    stock_repurchase        DECIMAL(20, 2),
    debt_repayment          DECIMAL(20, 2),
    
    -- Net change
    free_cash_flow          DECIMAL(20, 2) GENERATED ALWAYS AS (
                                COALESCE(operating_cash_flow, 0) - COALESCE(capital_expenditures, 0)
                            ) STORED,
    
    -- Audit
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_cashflow_company_period UNIQUE (company_id, period_end_date, period_type)
);

-- Indexes for cash flow statements
CREATE INDEX idx_cashflow_company ON cash_flow_statements(company_id);
CREATE INDEX idx_cashflow_period_date ON cash_flow_statements(period_end_date DESC);
CREATE INDEX idx_cashflow_company_period ON cash_flow_statements(company_id, period_end_date DESC);

-- Daily prices (partitioned by year)
CREATE TABLE daily_prices (
    id                  UUID DEFAULT gen_random_uuid(),
    company_id          UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    price_date          DATE NOT NULL,
    
    -- OHLC
    open                DECIMAL(15, 4),
    high                DECIMAL(15, 4),
    low                 DECIMAL(15, 4),
    close               DECIMAL(15, 4),
    adjusted_close      DECIMAL(15, 4),
    
    -- Volume & Corporate Actions
    volume              BIGINT,
    dividend_amount     DECIMAL(15, 4),
    split_coefficient   DECIMAL(10, 4),
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    PRIMARY KEY (id, price_date),
    CONSTRAINT uq_price_company_date UNIQUE (company_id, price_date)
) PARTITION BY RANGE (price_date);

-- Create partitions for daily_prices
CREATE TABLE daily_prices_2020 PARTITION OF daily_prices
    FOR VALUES FROM ('2020-01-01') TO ('2021-01-01');
CREATE TABLE daily_prices_2021 PARTITION OF daily_prices
    FOR VALUES FROM ('2021-01-01') TO ('2022-01-01');
CREATE TABLE daily_prices_2022 PARTITION OF daily_prices
    FOR VALUES FROM ('2022-01-01') TO ('2023-01-01');
CREATE TABLE daily_prices_2023 PARTITION OF daily_prices
    FOR VALUES FROM ('2023-01-01') TO ('2024-01-01');
CREATE TABLE daily_prices_2024 PARTITION OF daily_prices
    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');
CREATE TABLE daily_prices_2025 PARTITION OF daily_prices
    FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');
CREATE TABLE daily_prices_2026 PARTITION OF daily_prices
    FOR VALUES FROM ('2026-01-01') TO ('2027-01-01');
CREATE TABLE daily_prices_future PARTITION OF daily_prices
    FOR VALUES FROM ('2027-01-01') TO (MAXVALUE);

-- Indexes for daily_prices (applied to all partitions)
CREATE INDEX idx_prices_company_date ON daily_prices(company_id, price_date DESC);

-- ============================================================================
-- DERIVED METRICS TABLE
-- ============================================================================

-- Derived metrics (pre-computed for performance)
CREATE TABLE derived_metrics (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id              UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    period_end_date         DATE NOT NULL,
    period_type             VARCHAR(10) NOT NULL CHECK (period_type IN ('quarterly', 'annual')),
    
    -- Metric name and value (flexible schema for different metric types)
    metric_name             VARCHAR(100) NOT NULL,
    metric_value            DECIMAL(20, 6),
    
    -- Audit
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_derived_company_period_metric UNIQUE (company_id, period_end_date, period_type, metric_name)
);

-- Indexes for derived metrics
CREATE INDEX idx_derived_company ON derived_metrics(company_id);
CREATE INDEX idx_derived_company_period ON derived_metrics(company_id, period_end_date DESC);
CREATE INDEX idx_derived_metric_name ON derived_metrics(metric_name);
CREATE INDEX idx_derived_company_period_metric ON derived_metrics(company_id, period_end_date, metric_name);

-- ============================================================================
-- ANALYSIS TABLES
-- ============================================================================

-- Screeners
CREATE TABLE screeners (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Definition
    title               VARCHAR(100) NOT NULL,
    description         TEXT,
    filter_criteria     JSONB NOT NULL,
    sort_config         JSONB,
    display_columns     JSONB,
    
    -- Ordering
    display_order       INTEGER,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for screeners
CREATE INDEX idx_screeners_user ON screeners(user_id);
CREATE INDEX idx_screeners_filters ON screeners USING GIN (filter_criteria);

-- Verdicts
CREATE TABLE verdicts (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id              UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    user_id                 UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    
    -- Assessment
    final_verdict           VARCHAR(20) CHECK (final_verdict IN ('invest', 'pass', 'watchlist')),
    summary_text            TEXT,
    
    -- Structured assessment (JSONB for flexibility)
    strengths               JSONB,  -- Array of strength points
    weaknesses              JSONB,  -- Array of weakness points
    guidance_summary        TEXT,
    
    -- Version control
    lock_version            INTEGER NOT NULL DEFAULT 0,
    
    -- Audit
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_verdict_company_user UNIQUE (company_id, user_id)
);

-- Indexes for verdicts
CREATE INDEX idx_verdicts_user ON verdicts(user_id);
CREATE INDEX idx_verdicts_company ON verdicts(company_id);
CREATE INDEX idx_verdicts_final ON verdicts(final_verdict);

-- Verdict history (version tracking)
CREATE TABLE verdict_history (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    verdict_id          UUID NOT NULL REFERENCES verdicts(id) ON DELETE CASCADE,
    version             INTEGER NOT NULL,
    
    -- Snapshot of verdict at this version
    final_verdict       VARCHAR(20),
    summary_text        TEXT,
    
    -- Audit
    recorded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_verdict_version UNIQUE (verdict_id, version)
);

-- Indexes for verdict history
CREATE INDEX idx_verdict_history_verdict ON verdict_history(verdict_id);
CREATE INDEX idx_verdict_history_recorded ON verdict_history(recorded_at DESC);

-- ============================================================================
-- DOCUMENT TABLES
-- ============================================================================

-- Documents (stored in S3, metadata in DB)
CREATE TABLE documents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id          UUID NOT NULL REFERENCES companies(id) ON DELETE CASCADE,
    
    -- Document metadata
    document_type       VARCHAR(50) NOT NULL CHECK (document_type IN (
                            'investor_presentation',
                            'earnings_call_transcript',
                            'earnings_release',
                            'quarterly_report',
                            'annual_report',
                            'other'
                        )),
    period_end_date     DATE,
    title               VARCHAR(255) NOT NULL,
    
    -- Storage
    storage_key         VARCHAR(500) NOT NULL,  -- S3 key
    source_url          TEXT,
    
    -- File metadata
    file_size           BIGINT,
    mime_type           VARCHAR(100),
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_document_storage_key UNIQUE (storage_key)
);

-- Indexes for documents
CREATE INDEX idx_documents_company ON documents(company_id);
CREATE INDEX idx_documents_company_period ON documents(company_id, period_end_date DESC);
CREATE INDEX idx_documents_type ON documents(document_type);

-- Analysis reports (user-uploaded documents linked to verdicts)
CREATE TABLE analysis_reports (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    verdict_id          UUID NOT NULL REFERENCES verdicts(id) ON DELETE CASCADE,
    verdict_history_id  UUID REFERENCES verdict_history(id) ON DELETE SET NULL,
    
    -- Storage
    storage_key         VARCHAR(500) NOT NULL,
    filename            VARCHAR(255) NOT NULL,
    
    -- Audit
    uploaded_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_analysis_report_storage_key UNIQUE (storage_key)
);

-- Indexes for analysis reports
CREATE INDEX idx_analysis_reports_verdict ON analysis_reports(verdict_id);
CREATE INDEX idx_analysis_reports_uploaded ON analysis_reports(uploaded_at DESC);

-- ============================================================================
-- SYSTEM TABLES
-- ============================================================================

-- Schema migrations tracking
CREATE TABLE schema_migrations (
    version             INTEGER PRIMARY KEY,
    name                VARCHAR(255) NOT NULL,
    applied_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    execution_time_ms   INTEGER,
    checksum            VARCHAR(64)
);

-- Insert initial migration record
INSERT INTO schema_migrations (version, name, checksum) 
VALUES (1, '001_initial_schema', 'initial');

-- Background jobs tracking
CREATE TABLE background_jobs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type            VARCHAR(50) NOT NULL,
    
    -- Job timing
    scheduled_at        TIMESTAMPTZ NOT NULL,
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    
    -- Status
    status              VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN (
                            'pending', 'running', 'completed', 'failed', 'cancelled'
                        )),
    
    -- Results
    companies_processed INTEGER,
    records_updated     INTEGER,
    error_message       TEXT,
    error_details       JSONB,
    
    -- Audit
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for background jobs
CREATE INDEX idx_jobs_type_status ON background_jobs(job_type, status);
CREATE INDEX idx_jobs_scheduled ON background_jobs(scheduled_at DESC);
CREATE INDEX idx_jobs_status ON background_jobs(status);

-- ============================================================================
-- FUNCTIONS AND TRIGGERS
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply updated_at trigger to relevant tables
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_companies_updated_at BEFORE UPDATE ON companies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_screeners_updated_at BEFORE UPDATE ON screeners
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_verdicts_updated_at BEFORE UPDATE ON verdicts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_documents_updated_at BEFORE UPDATE ON documents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_user_preferences_updated_at BEFORE UPDATE ON user_preferences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_currencies_updated_at BEFORE UPDATE ON currencies
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_exchanges_updated_at BEFORE UPDATE ON exchanges
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_sectors_updated_at BEFORE UPDATE ON sectors
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE companies IS 'Master table for all companies in the platform';
COMMENT ON TABLE income_statements IS 'Quarterly and annual income statement data';
COMMENT ON TABLE balance_sheets IS 'Quarterly and annual balance sheet data';
COMMENT ON TABLE cash_flow_statements IS 'Quarterly and annual cash flow statement data';
COMMENT ON TABLE daily_prices IS 'Historical daily price data (partitioned by year)';
COMMENT ON TABLE derived_metrics IS 'Pre-computed metrics for screener performance';
COMMENT ON TABLE verdicts IS 'User investment verdicts for companies';
COMMENT ON TABLE screeners IS 'User-defined stock screeners';
COMMENT ON TABLE documents IS 'Company documents stored in S3';

-- ============================================================================
-- COMPLETION
-- ============================================================================

-- Grant appropriate permissions (adjust as needed for your setup)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO backend_user;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO backend_user;
