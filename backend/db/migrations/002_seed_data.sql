-- Migration: 002_seed_data.sql
-- Description: Seed data for development and testing
-- Date: 2026-01-17
-- Reference: docs/build-plan-v3/04-database-foundation.md

-- ============================================================================
-- WARNING: THIS IS DEVELOPMENT SEED DATA ONLY
-- DO NOT USE IN PRODUCTION
-- ============================================================================

-- ============================================================================
-- TEST USER
-- ============================================================================

-- Test User
-- Plain password: TestPass123! (for dev reference only)
-- This hash is generated using Argon2id with standard parameters
INSERT INTO users (id, username, email, password_hash, display_name, timezone, is_active, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001'::UUID,
    'testuser',
    'test@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$YXNkZmFzZGZhc2RmYXNkZg$VhlVhJ6sNgcKbmKy6m7t7nDJPDqKdD7K9QKdXJqKdXI',  -- TestPass123!
    'Test User',
    'Asia/Kolkata',
    TRUE,
    NOW(),
    NOW()
);

-- User preferences for test user
INSERT INTO user_preferences (id, user_id, document_row_order, default_period_count, default_period_type, theme, updated_at)
VALUES (
    gen_random_uuid(),
    '00000000-0000-0000-0000-000000000001'::UUID,
    '["investor_presentation", "earnings_call_transcript", "earnings_release"]'::JSONB,
    4,
    'quarterly',
    'light',
    NOW()
);

-- ============================================================================
-- ADDITIONAL REFERENCE DATA (extending what's in 001_initial_schema.sql)
-- ============================================================================

-- Note: Currencies and exchanges are already populated in initial schema
-- We're adding trading_days JSON to exchanges here

-- Update exchanges with trading_days information
UPDATE exchanges 
SET trading_days = '{
    "days": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
    "market_open": "09:30",
    "market_close": "16:00"
}'::JSONB
WHERE code IN ('NASDAQ', 'NYSE');

UPDATE exchanges 
SET trading_days = '{
    "days": ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday"],
    "market_open": "09:15",
    "market_close": "15:30"
}'::JSONB
WHERE code = 'BSE';

-- ============================================================================
-- SAMPLE US COMPANIES
-- ============================================================================

-- Apple Inc.
INSERT INTO companies (id, symbol, exchange, name, sector_id, industry, country, market_cap, currency, fiscal_year_end_month, description, cik, is_active, created_at, updated_at, latest_quarter)
VALUES (
    '10000000-0000-0000-0000-000000000001'::UUID,
    'AAPL',
    'NASDAQ',
    'Apple Inc.',
    (SELECT id FROM sectors WHERE name = 'Technology'),
    'Consumer Electronics',
    'USA',
    3000000000000,  -- $3T market cap
    'USD',
    9,  -- September fiscal year end
    'Apple Inc. designs, manufactures, and markets smartphones, personal computers, tablets, wearables, and accessories worldwide.',
    '0000320193',
    TRUE,
    NOW(),
    NOW(),
    '2024-09-30'
);

-- Microsoft Corporation
INSERT INTO companies (id, symbol, exchange, name, sector_id, industry, country, market_cap, currency, fiscal_year_end_month, description, cik, is_active, created_at, updated_at, latest_quarter)
VALUES (
    '10000000-0000-0000-0000-000000000002'::UUID,
    'MSFT',
    'NASDAQ',
    'Microsoft Corporation',
    (SELECT id FROM sectors WHERE name = 'Technology'),
    'Software—Infrastructure',
    'USA',
    2800000000000,  -- $2.8T market cap
    'USD',
    6,  -- June fiscal year end
    'Microsoft Corporation develops and supports software, services, devices and solutions worldwide.',
    '0000789019',
    TRUE,
    NOW(),
    NOW(),
    '2024-06-30'
);

-- JPMorgan Chase & Co.
INSERT INTO companies (id, symbol, exchange, name, sector_id, industry, country, market_cap, currency, fiscal_year_end_month, description, cik, is_active, created_at, updated_at, latest_quarter)
VALUES (
    '10000000-0000-0000-0000-000000000003'::UUID,
    'JPM',
    'NYSE',
    'JPMorgan Chase & Co.',
    (SELECT id FROM sectors WHERE name = 'Financials'),
    'Banks—Diversified',
    'USA',
    550000000000,  -- $550B market cap
    'USD',
    12,  -- December fiscal year end
    'JPMorgan Chase & Co. operates as a financial services company worldwide.',
    '0000019617',
    TRUE,
    NOW(),
    NOW(),
    '2024-12-31'
);

-- Johnson & Johnson
INSERT INTO companies (id, symbol, exchange, name, sector_id, industry, country, market_cap, currency, fiscal_year_end_month, description, cik, is_active, created_at, updated_at, latest_quarter)
VALUES (
    '10000000-0000-0000-0000-000000000004'::UUID,
    'JNJ',
    'NYSE',
    'Johnson & Johnson',
    (SELECT id FROM sectors WHERE name = 'Healthcare'),
    'Drug Manufacturers—General',
    'USA',
    400000000000,  -- $400B market cap
    'USD',
    12,  -- December fiscal year end
    'Johnson & Johnson researches, develops, manufactures, and sells various products in the healthcare field worldwide.',
    '0000200406',
    TRUE,
    NOW(),
    NOW(),
    '2024-12-31'
);

-- Tesla Inc.
INSERT INTO companies (id, symbol, exchange, name, sector_id, industry, country, market_cap, currency, fiscal_year_end_month, description, cik, is_active, created_at, updated_at, latest_quarter)
VALUES (
    '10000000-0000-0000-0000-000000000005'::UUID,
    'TSLA',
    'NASDAQ',
    'Tesla, Inc.',
    (SELECT id FROM sectors WHERE name = 'Consumer Cyclical'),
    'Auto Manufacturers',
    'USA',
    800000000000,  -- $800B market cap
    'USD',
    12,  -- December fiscal year end
    'Tesla, Inc. designs, develops, manufactures, leases, and sells electric vehicles, and energy generation and storage systems.',
    '0001318605',
    TRUE,
    NOW(),
    NOW(),
    '2024-12-31'
);

-- ============================================================================
-- SAMPLE FINANCIAL DATA FOR APPLE (AAPL)
-- 4 quarters of 2024 data
-- ============================================================================

-- Income Statements for Apple - Q1 2024 (Dec 2023)
INSERT INTO income_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_revenue, cost_of_revenue, gross_profit, operating_expenses, operating_income, interest_income, interest_expense, income_before_tax, income_tax_expense, net_income, depreciation_amortization, ebit, ebitda, basic_eps, diluted_eps, shares_outstanding, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2023-12-30',
    'quarterly',
    2024,
    1,
    119575000000,  -- $119.58B revenue
    66822000000,
    52753000000,
    14313000000,
    38440000000,
    1200000000,
    800000000,
    38840000000,
    6590000000,
    32250000000,  -- Net income
    2800000000,
    39240000000,
    42040000000,
    2.09,
    2.07,
    15550000000,  -- 15.55B shares outstanding
    NOW()
);

-- Income Statements for Apple - Q2 2024 (Mar 2024)
INSERT INTO income_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_revenue, cost_of_revenue, gross_profit, operating_expenses, operating_income, interest_income, interest_expense, income_before_tax, income_tax_expense, net_income, depreciation_amortization, ebit, ebitda, basic_eps, diluted_eps, shares_outstanding, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-03-30',
    'quarterly',
    2024,
    2,
    90753000000,  -- $90.75B revenue
    50850000000,
    39903000000,
    13637000000,
    26266000000,
    1100000000,
    750000000,
    26616000000,
    4513000000,
    22103000000,  -- Net income
    2700000000,
    27016000000,
    29716000000,
    1.43,
    1.42,
    15470000000,  -- 15.47B shares
    NOW()
);

-- Income Statements for Apple - Q3 2024 (Jun 2024)
INSERT INTO income_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_revenue, cost_of_revenue, gross_profit, operating_expenses, operating_income, interest_income, interest_expense, income_before_tax, income_tax_expense, net_income, depreciation_amortization, ebit, ebitda, basic_eps, diluted_eps, shares_outstanding, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-06-29',
    'quarterly',
    2024,
    3,
    85777000000,  -- $85.78B revenue
    48000000000,
    37777000000,
    13421000000,
    24356000000,
    1050000000,
    700000000,
    24706000000,
    4184000000,
    20522000000,  -- Net income
    2650000000,
    25056000000,
    27706000000,
    1.33,
    1.32,
    15410000000,  -- 15.41B shares
    NOW()
);

-- Income Statements for Apple - Q4 2024 (Sep 2024)
INSERT INTO income_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_revenue, cost_of_revenue, gross_profit, operating_expenses, operating_income, interest_income, interest_expense, income_before_tax, income_tax_expense, net_income, depreciation_amortization, ebit, ebitda, basic_eps, diluted_eps, shares_outstanding, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-09-28',
    'quarterly',
    2024,
    4,
    94930000000,  -- $94.93B revenue
    53100000000,
    41830000000,
    14100000000,
    27730000000,
    1150000000,
    780000000,
    28100000000,
    4760000000,
    23340000000,  -- Net income
    2750000000,
    28880000000,
    31630000000,
    1.52,
    1.51,
    15350000000,  -- 15.35B shares
    NOW()
);

-- Balance Sheets for Apple - Q1 2024 (Dec 2023)
INSERT INTO balance_sheets (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_assets, current_assets, cash_and_equivalents, short_term_investments, inventory, accounts_receivable, non_current_assets, property_plant_equipment, goodwill, intangible_assets, total_liabilities, current_liabilities, accounts_payable, short_term_debt, non_current_liabilities, long_term_debt, total_equity, retained_earnings, common_stock, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2023-12-30',
    'quarterly',
    2024,
    1,
    352755000000,  -- Total assets
    143566000000,
    29965000000,  -- Cash
    31590000000,  -- Short-term investments
    6511000000,
    60932000000,
    209189000000,
    43715000000,
    0,
    0,
    290020000000,  -- Total liabilities
    133973000000,
    62611000000,
    15000000000,  -- Short-term debt
    156047000000,
    106000000000,  -- Long-term debt
    62735000000,  -- Total equity
    -214000000,
    73812000000,
    NOW()
);

-- Balance Sheets for Apple - Q2 2024 (Mar 2024)
INSERT INTO balance_sheets (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_assets, current_assets, cash_and_equivalents, short_term_investments, inventory, accounts_receivable, non_current_assets, property_plant_equipment, goodwill, intangible_assets, total_liabilities, current_liabilities, accounts_payable, short_term_debt, non_current_liabilities, long_term_debt, total_equity, retained_earnings, common_stock, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-03-30',
    'quarterly',
    2024,
    2,
    337158000000,
    135000000000,
    24300000000,
    35830000000,
    5980000000,
    52000000000,
    202158000000,
    43800000000,
    0,
    0,
    279414000000,
    125000000000,
    58000000000,
    13500000000,
    154414000000,
    98000000000,
    57744000000,
    -3500000000,
    71900000000,
    NOW()
);

-- Balance Sheets for Apple - Q3 2024 (Jun 2024)
INSERT INTO balance_sheets (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_assets, current_assets, cash_and_equivalents, short_term_investments, inventory, accounts_receivable, non_current_assets, property_plant_equipment, goodwill, intangible_assets, total_liabilities, current_liabilities, accounts_payable, short_term_debt, non_current_liabilities, long_term_debt, total_equity, retained_earnings, common_stock, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-06-29',
    'quarterly',
    2024,
    3,
    341658000000,
    136500000000,
    27800000000,
    32900000000,
    6200000000,
    54300000000,
    205158000000,
    44100000000,
    0,
    0,
    281000000000,
    127000000000,
    59500000000,
    12800000000,
    154000000000,
    95500000000,
    60658000000,
    -1800000000,
    72100000000,
    NOW()
);

-- Balance Sheets for Apple - Q4 2024 (Sep 2024)
INSERT INTO balance_sheets (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, total_assets, current_assets, cash_and_equivalents, short_term_investments, inventory, accounts_receivable, non_current_assets, property_plant_equipment, goodwill, intangible_assets, total_liabilities, current_liabilities, accounts_payable, short_term_debt, non_current_liabilities, long_term_debt, total_equity, retained_earnings, common_stock, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-09-28',
    'quarterly',
    2024,
    4,
    364980000000,
    143700000000,
    29900000000,
    35100000000,
    6900000000,
    57400000000,
    221280000000,
    44500000000,
    0,
    0,
    308030000000,
    137000000000,
    64200000000,
    13200000000,
    171030000000,
    97000000000,
    56950000000,
    3700000000,
    77500000000,
    NOW()
);

-- Cash Flow Statements for Apple - Q1 2024 (Dec 2023)
INSERT INTO cash_flow_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, operating_cash_flow, net_income, depreciation_depletion, change_in_receivables, change_in_inventory, change_in_payables, investing_cash_flow, capital_expenditures, investments, financing_cash_flow, dividend_payout, stock_repurchase, debt_repayment, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2023-12-30',
    'quarterly',
    2024,
    1,
    34500000000,  -- Operating cash flow
    32250000000,
    2800000000,
    -2100000000,
    450000000,
    1800000000,
    -3200000000,  -- Investing cash flow
    2800000000,  -- CapEx
    -5100000000,
    -28700000000,  -- Financing cash flow
    3900000000,  -- Dividends
    22000000000,  -- Buybacks
    2500000000,
    NOW()
);

-- Cash Flow Statements for Apple - Q2 2024 (Mar 2024)
INSERT INTO cash_flow_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, operating_cash_flow, net_income, depreciation_depletion, change_in_receivables, change_in_inventory, change_in_payables, investing_cash_flow, capital_expenditures, investments, financing_cash_flow, dividend_payout, stock_repurchase, debt_repayment, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-03-30',
    'quarterly',
    2024,
    2,
    24800000000,
    22103000000,
    2700000000,
    -1500000000,
    380000000,
    1600000000,
    -2900000000,
    2600000000,
    -4700000000,
    -21300000000,
    3850000000,
    16000000000,
    1200000000,
    NOW()
);

-- Cash Flow Statements for Apple - Q3 2024 (Jun 2024)
INSERT INTO cash_flow_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, operating_cash_flow, net_income, depreciation_depletion, change_in_receivables, change_in_inventory, change_in_payables, investing_cash_flow, capital_expenditures, investments, financing_cash_flow, dividend_payout, stock_repurchase, debt_repayment, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-06-29',
    'quarterly',
    2024,
    3,
    23200000000,
    20522000000,
    2650000000,
    -1800000000,
    420000000,
    1900000000,
    -2700000000,
    2550000000,
    -4500000000,
    -19800000000,
    3800000000,
    14500000000,
    1100000000,
    NOW()
);

-- Cash Flow Statements for Apple - Q4 2024 (Sep 2024)
INSERT INTO cash_flow_statements (id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter, operating_cash_flow, net_income, depreciation_depletion, change_in_receivables, change_in_inventory, change_in_payables, investing_cash_flow, capital_expenditures, investments, financing_cash_flow, dividend_payout, stock_repurchase, debt_repayment, created_at)
VALUES (
    gen_random_uuid(),
    '10000000-0000-0000-0000-000000000001'::UUID,
    '2024-09-28',
    'quarterly',
    2024,
    4,
    27500000000,
    23340000000,
    2750000000,
    -2200000000,
    500000000,
    2100000000,
    -3100000000,
    2700000000,
    -5200000000,
    -23700000000,
    3900000000,
    18000000000,
    1300000000,
    NOW()
);

-- ============================================================================
-- SAMPLE DERIVED METRICS FOR APPLE
-- ============================================================================

-- YoY Revenue Growth for Apple Q4 2024 (assuming Q4 2023 was $89.5B)
INSERT INTO derived_metrics (id, company_id, period_end_date, period_type, metric_name, metric_value, created_at)
VALUES 
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'yoy_revenue_growth_pct', 6.07, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'gross_margin_pct', 44.06, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'operating_margin_pct', 29.21, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'net_margin_pct', 24.59, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'fcf_revenue_pct', 26.14, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-09-28', 'quarterly', 'roc_pct', 41.00, NOW());

-- Metrics for Q3 2024
INSERT INTO derived_metrics (id, company_id, period_end_date, period_type, metric_name, metric_value, created_at)
VALUES 
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-06-29', 'quarterly', 'yoy_revenue_growth_pct', 4.87, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-06-29', 'quarterly', 'gross_margin_pct', 44.04, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-06-29', 'quarterly', 'operating_margin_pct', 28.39, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-06-29', 'quarterly', 'net_margin_pct', 23.92, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-06-29', 'quarterly', 'fcf_revenue_pct', 24.07, NOW());

-- Metrics for Q2 2024
INSERT INTO derived_metrics (id, company_id, period_end_date, period_type, metric_name, metric_value, created_at)
VALUES 
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-03-30', 'quarterly', 'yoy_revenue_growth_pct', -4.31, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-03-30', 'quarterly', 'gross_margin_pct', 43.96, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-03-30', 'quarterly', 'operating_margin_pct', 28.94, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-03-30', 'quarterly', 'net_margin_pct', 24.36, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2024-03-30', 'quarterly', 'fcf_revenue_pct', 24.42, NOW());

-- Metrics for Q1 2024
INSERT INTO derived_metrics (id, company_id, period_end_date, period_type, metric_name, metric_value, created_at)
VALUES 
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2023-12-30', 'quarterly', 'yoy_revenue_growth_pct', 2.07, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2023-12-30', 'quarterly', 'gross_margin_pct', 44.11, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2023-12-30', 'quarterly', 'operating_margin_pct', 32.15, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2023-12-30', 'quarterly', 'net_margin_pct', 26.97, NOW()),
    (gen_random_uuid(), '10000000-0000-0000-0000-000000000001'::UUID, '2023-12-30', 'quarterly', 'fcf_revenue_pct', 26.52, NOW());

-- ============================================================================
-- SAMPLE SCREENER
-- ============================================================================

-- High Growth Tech Screener
INSERT INTO screeners (id, user_id, title, description, filter_criteria, sort_config, display_columns, display_order, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    '00000000-0000-0000-0000-000000000001'::UUID,
    'High Growth Tech',
    'Large-cap technology companies with strong growth metrics',
    '{
        "exchanges": ["NASDAQ"],
        "sectors": ["Technology"],
        "market_cap": {
            "min": 100000000000,
            "currency": "USD"
        },
        "metrics": {
            "yoy_revenue_growth_pct": {
                "min": 0
            }
        }
    }'::JSONB,
    '{
        "column": "market_cap",
        "direction": "desc"
    }'::JSONB,
    '["symbol", "name", "market_cap", "yoy_revenue_growth_pct", "gross_margin_pct", "operating_margin_pct"]'::JSONB,
    1,
    NOW(),
    NOW()
);

-- Value Financials Screener
INSERT INTO screeners (id, user_id, title, description, filter_criteria, sort_config, display_columns, display_order, created_at, updated_at)
VALUES (
    gen_random_uuid(),
    '00000000-0000-0000-0000-000000000001'::UUID,
    'Value Financials',
    'Financial sector companies with attractive valuations',
    '{
        "exchanges": ["NYSE"],
        "sectors": ["Financials"],
        "market_cap": {
            "min": 50000000000,
            "currency": "USD"
        }
    }'::JSONB,
    '{
        "column": "market_cap",
        "direction": "desc"
    }'::JSONB,
    '["symbol", "name", "market_cap", "net_margin_pct", "roc_pct"]'::JSONB,
    2,
    NOW(),
    NOW()
);

-- ============================================================================
-- SEED DATA MIGRATION COMPLETE
-- ============================================================================

-- Update schema migrations
INSERT INTO schema_migrations (version, name, checksum) 
VALUES (2, '002_seed_data', 'seed_data_v1');

-- Summary of seed data
DO $$
BEGIN
    RAISE NOTICE '=================================================';
    RAISE NOTICE 'Seed Data Migration Complete';
    RAISE NOTICE '=================================================';
    RAISE NOTICE 'Test User: testuser / TestPass123!';
    RAISE NOTICE 'Companies: % records', (SELECT COUNT(*) FROM companies);
    RAISE NOTICE 'Income Statements: % records', (SELECT COUNT(*) FROM income_statements);
    RAISE NOTICE 'Balance Sheets: % records', (SELECT COUNT(*) FROM balance_sheets);
    RAISE NOTICE 'Cash Flows: % records', (SELECT COUNT(*) FROM cash_flow_statements);
    RAISE NOTICE 'Derived Metrics: % records', (SELECT COUNT(*) FROM derived_metrics);
    RAISE NOTICE 'Screeners: % records', (SELECT COUNT(*) FROM screeners);
    RAISE NOTICE '=================================================';
END $$;
