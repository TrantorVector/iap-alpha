/// Company repository for company and financial data
use crate::models::{
    BalanceSheet, CashFlowStatement, Company, DailyPrice, DerivedMetric, IncomeStatement,
};
use crate::{DbError, DbResult};
use chrono::NaiveDate;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

// =============================================================================
// DTOs (Data Transfer Objects)
// =============================================================================

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: i64,
    pub offset: i64,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            limit: 50,
            offset: 0,
        }
    }
}

/// Company filters for list queries
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompanyFilters {
    pub exchange: Option<String>,
    pub country: Option<String>,
    pub sector_id: Option<Uuid>,
    pub is_active: Option<bool>,
}

/// Income statement insert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeStatementInsert {
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    pub total_revenue: Option<BigDecimal>,
    pub cost_of_revenue: Option<BigDecimal>,
    pub gross_profit: Option<BigDecimal>,
    pub operating_expenses: Option<BigDecimal>,
    pub operating_income: Option<BigDecimal>,
    pub interest_income: Option<BigDecimal>,
    pub interest_expense: Option<BigDecimal>,
    pub income_before_tax: Option<BigDecimal>,
    pub income_tax_expense: Option<BigDecimal>,
    pub net_income: Option<BigDecimal>,
    pub depreciation_amortization: Option<BigDecimal>,
    pub ebit: Option<BigDecimal>,
    pub ebitda: Option<BigDecimal>,
    pub basic_eps: Option<BigDecimal>,
    pub diluted_eps: Option<BigDecimal>,
    pub shares_outstanding: Option<i64>,
}

/// Balance sheet insert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSheetInsert {
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    pub total_assets: Option<BigDecimal>,
    pub current_assets: Option<BigDecimal>,
    pub cash_and_equivalents: Option<BigDecimal>,
    pub short_term_investments: Option<BigDecimal>,
    pub inventory: Option<BigDecimal>,
    pub accounts_receivable: Option<BigDecimal>,
    pub non_current_assets: Option<BigDecimal>,
    pub property_plant_equipment: Option<BigDecimal>,
    pub goodwill: Option<BigDecimal>,
    pub intangible_assets: Option<BigDecimal>,
    pub total_liabilities: Option<BigDecimal>,
    pub current_liabilities: Option<BigDecimal>,
    pub accounts_payable: Option<BigDecimal>,
    pub short_term_debt: Option<BigDecimal>,
    pub non_current_liabilities: Option<BigDecimal>,
    pub long_term_debt: Option<BigDecimal>,
    pub total_equity: Option<BigDecimal>,
    pub retained_earnings: Option<BigDecimal>,
    pub common_stock: Option<BigDecimal>,
}

/// Cash flow statement insert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowStatementInsert {
    pub company_id: Uuid,
    pub period_end_date: NaiveDate,
    pub period_type: String,
    pub fiscal_year: Option<i32>,
    pub fiscal_quarter: Option<i32>,
    pub operating_cash_flow: Option<BigDecimal>,
    pub net_income: Option<BigDecimal>,
    pub depreciation_depletion: Option<BigDecimal>,
    pub change_in_receivables: Option<BigDecimal>,
    pub change_in_inventory: Option<BigDecimal>,
    pub change_in_payables: Option<BigDecimal>,
    pub investing_cash_flow: Option<BigDecimal>,
    pub capital_expenditures: Option<BigDecimal>,
    pub investments: Option<BigDecimal>,
    pub financing_cash_flow: Option<BigDecimal>,
    pub dividend_payout: Option<BigDecimal>,
    pub stock_repurchase: Option<BigDecimal>,
    pub debt_repayment: Option<BigDecimal>,
}

/// Daily price insert data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPriceInsert {
    pub company_id: Uuid,
    pub price_date: NaiveDate,
    pub open: Option<BigDecimal>,
    pub high: Option<BigDecimal>,
    pub low: Option<BigDecimal>,
    pub close: Option<BigDecimal>,
    pub adjusted_close: Option<BigDecimal>,
    pub volume: Option<i64>,
    pub dividend_amount: Option<BigDecimal>,
    pub split_coefficient: Option<BigDecimal>,
}

// =============================================================================
// Company Repository
// =============================================================================

/// Company repository
pub struct CompanyRepository {
    pool: PgPool,
}

impl CompanyRepository {
    /// Create a new company repository
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // =========================================================================
    // Company Query Methods
    // =========================================================================

    /// Find company by ID
    pub async fn find_by_id(&self, id: Uuid) -> DbResult<Option<Company>> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            SELECT id, symbol, exchange, name, sector_id, industry, country,
                   market_cap, currency, fiscal_year_end_month, description, cik,
                   address, latest_quarter, is_active, created_at, updated_at
            FROM companies
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(company)
    }

    /// Find company by symbol and exchange
    pub async fn find_by_symbol(&self, symbol: &str, exchange: &str) -> DbResult<Option<Company>> {
        let company = sqlx::query_as::<_, Company>(
            r#"
            SELECT id, symbol, exchange, name, sector_id, industry, country,
                   market_cap, currency, fiscal_year_end_month, description, cik,
                   address, latest_quarter, is_active, created_at, updated_at
            FROM companies
            WHERE symbol = $1 AND exchange = $2
            "#,
        )
        .bind(symbol)
        .bind(exchange)
        .fetch_optional(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(company)
    }

    /// List companies with filters and pagination
    pub async fn list(
        &self,
        filters: CompanyFilters,
        pagination: Pagination,
    ) -> DbResult<Vec<Company>> {
        let mut query = String::from(
            r#"
            SELECT id, symbol, exchange, name, sector_id, industry, country,
                   market_cap, currency, fiscal_year_end_month, description, cik,
                   address, latest_quarter, is_active, created_at, updated_at
            FROM companies
            WHERE 1=1
            "#,
        );

        // Build dynamic WHERE clause
        if filters.exchange.is_some() {
            query.push_str(" AND exchange = $3");
        }
        if filters.country.is_some() {
            query.push_str(" AND country = $4");
        }
        if filters.sector_id.is_some() {
            query.push_str(" AND sector_id = $5");
        }
        if let Some(is_active) = filters.is_active {
            if is_active {
                query.push_str(" AND is_active = true");
            }
        }

        query.push_str(" ORDER BY market_cap DESC NULLS LAST LIMIT $1 OFFSET $2");

        let mut q = sqlx::query_as::<_, Company>(&query)
            .bind(pagination.limit)
            .bind(pagination.offset);

        if let Some(ref exchange) = filters.exchange {
            q = q.bind(exchange);
        }
        if let Some(ref country) = filters.country {
            q = q.bind(country);
        }
        if let Some(sector_id) = filters.sector_id {
            q = q.bind(sector_id);
        }

        let companies = q.fetch_all(&self.pool).await.map_err(DbError::from)?;

        Ok(companies)
    }

    /// Full-text search companies
    pub async fn search(&self, query: &str, limit: i32) -> DbResult<Vec<Company>> {
        let companies = sqlx::query_as::<_, Company>(
            r#"
            SELECT id, symbol, exchange, name, sector_id, industry, country,
                   market_cap, currency, fiscal_year_end_month, description, cik,
                   address, latest_quarter, is_active, created_at, updated_at
            FROM companies
            WHERE search_vector @@ plainto_tsquery('english', $1)
            ORDER BY ts_rank(search_vector, plainto_tsquery('english', $1)) DESC
            LIMIT $2
            "#,
        )
        .bind(query)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(companies)
    }

    // =========================================================================
    // Financial Data Query Methods
    // =========================================================================

    /// Get income statements for a company
    pub async fn get_income_statements(
        &self,
        company_id: Uuid,
        period_type: &str,
        limit: i32,
    ) -> DbResult<Vec<IncomeStatement>> {
        let statements = sqlx::query_as::<_, IncomeStatement>(
            r#"
            SELECT id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                   total_revenue, cost_of_revenue, gross_profit, operating_expenses,
                   operating_income, interest_income, interest_expense, income_before_tax,
                   income_tax_expense, net_income, depreciation_amortization, ebit, ebitda,
                   basic_eps, diluted_eps, shares_outstanding, created_at
            FROM income_statements
            WHERE company_id = $1 AND period_type = $2
            ORDER BY period_end_date DESC
            LIMIT $3
            "#,
        )
        .bind(company_id)
        .bind(period_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(statements)
    }

    /// Get balance sheets for a company
    pub async fn get_balance_sheets(
        &self,
        company_id: Uuid,
        period_type: &str,
        limit: i32,
    ) -> DbResult<Vec<BalanceSheet>> {
        let sheets = sqlx::query_as::<_, BalanceSheet>(
            r#"
            SELECT id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                   total_assets, current_assets, cash_and_equivalents, short_term_investments,
                   inventory, accounts_receivable, non_current_assets, property_plant_equipment,
                   goodwill, intangible_assets, total_liabilities, current_liabilities,
                   accounts_payable, short_term_debt, non_current_liabilities, long_term_debt,
                   total_equity, retained_earnings, common_stock, total_debt, net_debt, created_at
            FROM balance_sheets
            WHERE company_id = $1 AND period_type = $2
            ORDER BY period_end_date DESC
            LIMIT $3
            "#,
        )
        .bind(company_id)
        .bind(period_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(sheets)
    }

    /// Get cash flow statements for a company
    pub async fn get_cash_flow_statements(
        &self,
        company_id: Uuid,
        period_type: &str,
        limit: i32,
    ) -> DbResult<Vec<CashFlowStatement>> {
        let statements = sqlx::query_as::<_, CashFlowStatement>(
            r#"
            SELECT id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                   operating_cash_flow, net_income, depreciation_depletion, change_in_receivables,
                   change_in_inventory, change_in_payables, investing_cash_flow,
                   capital_expenditures, investments, financing_cash_flow, dividend_payout,
                   stock_repurchase, debt_repayment, free_cash_flow, created_at
            FROM cash_flow_statements
            WHERE company_id = $1 AND period_type = $2
            ORDER BY period_end_date DESC
            LIMIT $3
            "#,
        )
        .bind(company_id)
        .bind(period_type)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(statements)
    }

    /// Get daily prices for a company within a date range
    pub async fn get_daily_prices(
        &self,
        company_id: Uuid,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> DbResult<Vec<DailyPrice>> {
        let prices = sqlx::query_as::<_, DailyPrice>(
            r#"
            SELECT id, company_id, price_date, open, high, low, close, adjusted_close,
                   volume, dividend_amount, split_coefficient, created_at
            FROM daily_prices
            WHERE company_id = $1
              AND price_date >= $2
              AND price_date <= $3
            ORDER BY price_date ASC
            "#,
        )
        .bind(company_id)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(prices)
    }

    // =========================================================================
    // Derived Metrics Query Methods
    // =========================================================================

    /// Get derived metrics for a company
    pub async fn get_derived_metrics(
        &self,
        company_id: Uuid,
        period_type: &str,
        metric_names: Vec<String>,
    ) -> DbResult<Vec<DerivedMetric>> {
        let metrics = sqlx::query_as::<_, DerivedMetric>(
            r#"
            SELECT id, company_id, period_end_date, period_type, metric_name, metric_value, created_at
            FROM derived_metrics
            WHERE company_id = $1
              AND period_type = $2
              AND metric_name = ANY($3)
            ORDER BY period_end_date DESC, metric_name ASC
            "#,
        )
        .bind(company_id)
        .bind(period_type)
        .bind(&metric_names)
        .fetch_all(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(metrics)
    }

    // =========================================================================
    // Upsert Methods (for background job data insertion)
    // =========================================================================

    /// Upsert income statement
    pub async fn upsert_income_statement(
        &self,
        data: IncomeStatementInsert,
    ) -> DbResult<IncomeStatement> {
        let id = Uuid::new_v4();
        
        let statement = sqlx::query_as::<_, IncomeStatement>(
            r#"
            INSERT INTO income_statements (
                id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                total_revenue, cost_of_revenue, gross_profit, operating_expenses,
                operating_income, interest_income, interest_expense, income_before_tax,
                income_tax_expense, net_income, depreciation_amortization, ebit, ebitda,
                basic_eps, diluted_eps, shares_outstanding, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, NOW())
            ON CONFLICT (company_id, period_end_date, period_type) DO UPDATE SET
                fiscal_year = EXCLUDED.fiscal_year,
                fiscal_quarter = EXCLUDED.fiscal_quarter,
                total_revenue = EXCLUDED.total_revenue,
                cost_of_revenue = EXCLUDED.cost_of_revenue,
                gross_profit = EXCLUDED.gross_profit,
                operating_expenses = EXCLUDED.operating_expenses,
                operating_income = EXCLUDED.operating_income,
                interest_income = EXCLUDED.interest_income,
                interest_expense = EXCLUDED.interest_expense,
                income_before_tax = EXCLUDED.income_before_tax,
                income_tax_expense = EXCLUDED.income_tax_expense,
                net_income = EXCLUDED.net_income,
                depreciation_amortization = EXCLUDED.depreciation_amortization,
                ebit = EXCLUDED.ebit,
                ebitda = EXCLUDED.ebitda,
                basic_eps = EXCLUDED.basic_eps,
                diluted_eps = EXCLUDED.diluted_eps,
                shares_outstanding = EXCLUDED.shares_outstanding
            RETURNING id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                      total_revenue, cost_of_revenue, gross_profit, operating_expenses,
                      operating_income, interest_income, interest_expense, income_before_tax,
                      income_tax_expense, net_income, depreciation_amortization, ebit, ebitda,
                      basic_eps, diluted_eps, shares_outstanding, created_at
            "#,
        )
        .bind(id)
        .bind(data.company_id)
        .bind(data.period_end_date)
        .bind(data.period_type)
        .bind(data.fiscal_year)
        .bind(data.fiscal_quarter)
        .bind(data.total_revenue)
        .bind(data.cost_of_revenue)
        .bind(data.gross_profit)
        .bind(data.operating_expenses)
        .bind(data.operating_income)
        .bind(data.interest_income)
        .bind(data.interest_expense)
        .bind(data.income_before_tax)
        .bind(data.income_tax_expense)
        .bind(data.net_income)
        .bind(data.depreciation_amortization)
        .bind(data.ebit)
        .bind(data.ebitda)
        .bind(data.basic_eps)
        .bind(data.diluted_eps)
        .bind(data.shares_outstanding)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(statement)
    }

    /// Upsert balance sheet
    pub async fn upsert_balance_sheet(&self, data: BalanceSheetInsert) -> DbResult<BalanceSheet> {
        let id = Uuid::new_v4();
        
        let sheet = sqlx::query_as::<_, BalanceSheet>(
            r#"
            INSERT INTO balance_sheets (
                id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                total_assets, current_assets, cash_and_equivalents, short_term_investments,
                inventory, accounts_receivable, non_current_assets, property_plant_equipment,
                goodwill, intangible_assets, total_liabilities, current_liabilities,
                accounts_payable, short_term_debt, non_current_liabilities, long_term_debt,
                total_equity, retained_earnings, common_stock, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, NOW())
            ON CONFLICT (company_id, period_end_date, period_type) DO UPDATE SET
                fiscal_year = EXCLUDED.fiscal_year,
                fiscal_quarter = EXCLUDED.fiscal_quarter,
                total_assets = EXCLUDED.total_assets,
                current_assets = EXCLUDED.current_assets,
                cash_and_equivalents = EXCLUDED.cash_and_equivalents,
                short_term_investments = EXCLUDED.short_term_investments,
                inventory = EXCLUDED.inventory,
                accounts_receivable = EXCLUDED.accounts_receivable,
                non_current_assets = EXCLUDED.non_current_assets,
                property_plant_equipment = EXCLUDED.property_plant_equipment,
                goodwill = EXCLUDED.goodwill,
                intangible_assets = EXCLUDED.intangible_assets,
                total_liabilities = EXCLUDED.total_liabilities,
                current_liabilities = EXCLUDED.current_liabilities,
                accounts_payable = EXCLUDED.accounts_payable,
                short_term_debt = EXCLUDED.short_term_debt,
                non_current_liabilities = EXCLUDED.non_current_liabilities,
                long_term_debt = EXCLUDED.long_term_debt,
                total_equity = EXCLUDED.total_equity,
                retained_earnings = EXCLUDED.retained_earnings,
                common_stock = EXCLUDED.common_stock
            RETURNING id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                      total_assets, current_assets, cash_and_equivalents, short_term_investments,
                      inventory, accounts_receivable, non_current_assets, property_plant_equipment,
                      goodwill, intangible_assets, total_liabilities, current_liabilities,
                      accounts_payable, short_term_debt, non_current_liabilities, long_term_debt,
                      total_equity, retained_earnings, common_stock, total_debt, net_debt, created_at
            "#,
        )
        .bind(id)
        .bind(data.company_id)
        .bind(data.period_end_date)
        .bind(data.period_type)
        .bind(data.fiscal_year)
        .bind(data.fiscal_quarter)
        .bind(data.total_assets)
        .bind(data.current_assets)
        .bind(data.cash_and_equivalents)
        .bind(data.short_term_investments)
        .bind(data.inventory)
        .bind(data.accounts_receivable)
        .bind(data.non_current_assets)
        .bind(data.property_plant_equipment)
        .bind(data.goodwill)
        .bind(data.intangible_assets)
        .bind(data.total_liabilities)
        .bind(data.current_liabilities)
        .bind(data.accounts_payable)
        .bind(data.short_term_debt)
        .bind(data.non_current_liabilities)
        .bind(data.long_term_debt)
        .bind(data.total_equity)
        .bind(data.retained_earnings)
        .bind(data.common_stock)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(sheet)
    }

    /// Upsert cash flow statement
    pub async fn upsert_cash_flow_statement(
        &self,
        data: CashFlowStatementInsert,
    ) -> DbResult<CashFlowStatement> {
        let id = Uuid::new_v4();
        
        let statement = sqlx::query_as::<_, CashFlowStatement>(
            r#"
            INSERT INTO cash_flow_statements (
                id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                operating_cash_flow, net_income, depreciation_depletion, change_in_receivables,
                change_in_inventory, change_in_payables, investing_cash_flow,
                capital_expenditures, investments, financing_cash_flow, dividend_payout,
                stock_repurchase, debt_repayment, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, NOW())
            ON CONFLICT (company_id, period_end_date, period_type) DO UPDATE SET
                fiscal_year = EXCLUDED.fiscal_year,
                fiscal_quarter = EXCLUDED.fiscal_quarter,
                operating_cash_flow = EXCLUDED.operating_cash_flow,
                net_income = EXCLUDED.net_income,
                depreciation_depletion = EXCLUDED.depreciation_depletion,
                change_in_receivables = EXCLUDED.change_in_receivables,
                change_in_inventory = EXCLUDED.change_in_inventory,
                change_in_payables = EXCLUDED.change_in_payables,
                investing_cash_flow = EXCLUDED.investing_cash_flow,
                capital_expenditures = EXCLUDED.capital_expenditures,
                investments = EXCLUDED.investments,
                financing_cash_flow = EXCLUDED.financing_cash_flow,
                dividend_payout = EXCLUDED.dividend_payout,
                stock_repurchase = EXCLUDED.stock_repurchase,
                debt_repayment = EXCLUDED.debt_repayment
            RETURNING id, company_id, period_end_date, period_type, fiscal_year, fiscal_quarter,
                      operating_cash_flow, net_income, depreciation_depletion, change_in_receivables,
                      change_in_inventory, change_in_payables, investing_cash_flow,
                      capital_expenditures, investments, financing_cash_flow, dividend_payout,
                      stock_repurchase, debt_repayment, free_cash_flow, created_at
            "#,
        )
        .bind(id)
        .bind(data.company_id)
        .bind(data.period_end_date)
        .bind(data.period_type)
        .bind(data.fiscal_year)
        .bind(data.fiscal_quarter)
        .bind(data.operating_cash_flow)
        .bind(data.net_income)
        .bind(data.depreciation_depletion)
        .bind(data.change_in_receivables)
        .bind(data.change_in_inventory)
        .bind(data.change_in_payables)
        .bind(data.investing_cash_flow)
        .bind(data.capital_expenditures)
        .bind(data.investments)
        .bind(data.financing_cash_flow)
        .bind(data.dividend_payout)
        .bind(data.stock_repurchase)
        .bind(data.debt_repayment)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(statement)
    }

    /// Upsert daily price
    pub async fn upsert_daily_price(&self, data: DailyPriceInsert) -> DbResult<DailyPrice> {
        let id = Uuid::new_v4();
        
        let price = sqlx::query_as::<_, DailyPrice>(
            r#"
            INSERT INTO daily_prices (
                id, company_id, price_date, open, high, low, close, adjusted_close,
                volume, dividend_amount, split_coefficient, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW())
            ON CONFLICT (company_id, price_date) DO UPDATE SET
                open = EXCLUDED.open,
                high = EXCLUDED.high,
                low = EXCLUDED.low,
                close = EXCLUDED.close,
                adjusted_close = EXCLUDED.adjusted_close,
                volume = EXCLUDED.volume,
                dividend_amount = EXCLUDED.dividend_amount,
                split_coefficient = EXCLUDED.split_coefficient
            RETURNING id, company_id, price_date, open, high, low, close, adjusted_close,
                      volume, dividend_amount, split_coefficient, created_at
            "#,
        )
        .bind(id)
        .bind(data.company_id)
        .bind(data.price_date)
        .bind(data.open)
        .bind(data.high)
        .bind(data.low)
        .bind(data.close)
        .bind(data.adjusted_close)
        .bind(data.volume)
        .bind(data.dividend_amount)
        .bind(data.split_coefficient)
        .fetch_one(&self.pool)
        .await
        .map_err(DbError::from)?;

        Ok(price)
    }
}
