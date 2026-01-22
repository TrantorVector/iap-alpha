use async_trait::async_trait;
use bytes::Bytes;
use domain::domain::{
    BalanceSheet, CashFlowStatement, CompanyOverview, DailyPrice, EarningsEvent, IncomeStatement,
    OutputSize,
};
use domain::error::AppError;
use domain::ports::market_data::MarketDataProvider;
use domain::ports::storage::ObjectStorage;

use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Clone)]
pub struct MockMarketDataProvider {
    data_path: PathBuf,
    delay: Duration,
}

impl Default for MockMarketDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl MockMarketDataProvider {
    pub fn new() -> Self {
        let data_path = Self::find_golden_copy_path();
        let delay_ms = std::env::var("MOCK_API_DELAY_MS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100); // Default 100ms delay

        Self {
            data_path,
            delay: Duration::from_millis(delay_ms),
        }
    }

    fn find_golden_copy_path() -> PathBuf {
        if let Ok(path) = std::env::var("MOCK_DATA_PATH") {
            return PathBuf::from(path);
        }
        let possible_paths = vec![
            "golden-copy",
            "../golden-copy",
            "../../golden-copy",
            "/home/founder/iap-alpha/golden-copy",
        ];
        for p in possible_paths {
            if Path::new(p).exists() {
                return PathBuf::from(p);
            }
        }
        PathBuf::from("golden-copy")
    }

    async fn read_json<T: serde::de::DeserializeOwned>(
        &self,
        filename: &str,
    ) -> Result<T, AppError> {
        let path = self.data_path.join(filename);
        let content = tokio::fs::read(&path).await.map_err(|e| {
            AppError::InternalError(format!("Failed to read mock data file {:?}: {}", path, e))
        })?;
        serde_json::from_slice(&content).map_err(|e| {
            AppError::InternalError(format!("Failed to parse mock data file {:?}: {}", path, e))
        })
    }

    async fn simulate_delay(&self) {
        if self.delay.as_millis() > 0 {
            sleep(self.delay).await;
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MockIncomeStatementResponse {
    annual_reports: Vec<serde_json::Value>,
    quarterly_reports: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MockBalanceSheetResponse {
    annual_reports: Vec<serde_json::Value>,
    quarterly_reports: Vec<serde_json::Value>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct MockCashFlowResponse {
    annual_reports: Vec<serde_json::Value>,
    quarterly_reports: Vec<serde_json::Value>,
}

#[async_trait]
impl MarketDataProvider for MockMarketDataProvider {
    async fn get_company_overview(&self, symbol: &str) -> Result<CompanyOverview, AppError> {
        self.simulate_delay().await;
        // Load using helper to handle PascalCase keys
        let helper: MockCompanyOverviewResponse = self.read_json("overview-output.json").await?;

        let mut overview: CompanyOverview = helper.into();

        if symbol != "IBM" {
            overview.symbol = symbol.to_string();
            overview.name = match symbol {
                "AAPL" => "Apple Inc.".to_string(),
                "MSFT" => "Microsoft Corporation".to_string(),
                _ => format!("Mock Company {}", symbol),
            };
        }

        Ok(overview)
    }

    async fn get_income_statement(&self, _symbol: &str) -> Result<Vec<IncomeStatement>, AppError> {
        self.simulate_delay().await;
        // Note: filename typo in golden copy "inome"
        let response: MockIncomeStatementResponse =
            self.read_json("inome-statement-output.json").await?;

        // Convert to domain types
        // Simple mapping: serialize back to Value then deserialize to domain type or manual mapping.
        // Domain types use camelCase or snake_case? Domain uses snake_case, JSON uses camelCase.
        // We rely on serde renaming or we need to map manually.
        // Since we didn't add serde(rename_all) to domain types (we just added fields),
        // we might have issues if keys don't match.
        // CompanyOverview matched because keys were explicit.
        // For IncomeStatement, let's try direct deserialization with a helper or manual map.
        // The domain fields are like `gross_profit`, JSON is `grossProfit`.
        // We need a helper struct to deserialize camelCase then convert.

        let mut statements = Vec::new();
        for report in response.annual_reports {
            // Use annual reports
            // Use serde_json::from_value with a helper struct that has [serde(rename_all = "camelCase")]
            let helper: IncomeStatementHelper = serde_json::from_value(report)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            statements.push(helper.into());
        }

        Ok(statements)
    }

    async fn get_balance_sheet(&self, _symbol: &str) -> Result<Vec<BalanceSheet>, AppError> {
        self.simulate_delay().await;
        let response: MockBalanceSheetResponse =
            self.read_json("balance-sheet-output.json").await?;
        let mut sheets = Vec::new();
        for report in response.annual_reports {
            let helper: BalanceSheetHelper = serde_json::from_value(report)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            sheets.push(helper.into());
        }
        Ok(sheets)
    }

    async fn get_cash_flow(&self, _symbol: &str) -> Result<Vec<CashFlowStatement>, AppError> {
        self.simulate_delay().await;
        let response: MockCashFlowResponse = self.read_json("cash-flow-output.json").await?;
        let mut flows = Vec::new();
        for report in response.annual_reports {
            let helper: CashFlowHelper = serde_json::from_value(report)
                .map_err(|e| AppError::InternalError(e.to_string()))?;
            flows.push(helper.into());
        }
        Ok(flows)
    }

    async fn get_daily_prices(
        &self,
        _symbol: &str,
        _output_size: OutputSize,
    ) -> Result<Vec<DailyPrice>, AppError> {
        self.simulate_delay().await;
        let json: serde_json::Value = self
            .read_json("time-series-daily-adjusted-output.json")
            .await?;

        let time_series = json
            .get("Time Series (Daily)")
            .ok_or_else(|| AppError::InternalError("Missing Time Series (Daily)".into()))?;
        let time_series = time_series
            .as_object()
            .ok_or_else(|| AppError::InternalError("Time Series is not an object".into()))?;

        let mut prices = Vec::new();
        for (date_str, values) in time_series {
            let date = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|e| {
                AppError::InternalError(format!("Failed to parse date {}: {}", date_str, e))
            })?;

            let open = values["1. open"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let high = values["2. high"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let low = values["3. low"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            let close = values["4. close"]
                .as_str()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);

            prices.push(DailyPrice {
                date,
                open,
                high,
                low,
                close,
            });
        }

        prices.sort_by_key(|p| p.date);
        Ok(prices)
    }

    async fn get_earnings_calendar(&self) -> Result<Vec<EarningsEvent>, AppError> {
        self.simulate_delay().await;
        let path = self.data_path.join("earnings-calendar-output.csv");
        let content = tokio::fs::read_to_string(&path).await.map_err(|e| {
            AppError::InternalError(format!("Failed to read mock data file {:?}: {}", path, e))
        })?;

        let mut events = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if i == 0 {
                continue;
            } // Skip header
            let parts: Vec<&str> = line.split(',').collect::<Vec<&str>>();
            if parts.len() >= 3 {
                let symbol = parts[0].to_string();
                let name = parts[1].to_string();
                let report_date_str = parts[2];
                if let Ok(report_date) =
                    chrono::NaiveDate::parse_from_str(report_date_str, "%Y-%m-%d")
                {
                    // Basic mapping
                    let fiscal_date_ending = if parts.len() > 3 && !parts[3].is_empty() {
                        chrono::NaiveDate::parse_from_str(parts[3], "%Y-%m-%d").ok()
                    } else {
                        None
                    };

                    let estimate = if parts.len() > 4 && !parts[4].is_empty() {
                        parts[4].parse().ok()
                    } else {
                        None
                    };

                    let currency = if parts.len() > 5 && !parts[5].is_empty() {
                        Some(parts[5].to_string())
                    } else {
                        None
                    };

                    events.push(EarningsEvent {
                        symbol,
                        name,
                        report_date,
                        fiscal_date_ending,
                        estimate,
                        currency,
                    });
                }
            }
        }
        Ok(events)
    }
}

// Helper structs for mapping JSON camelCase to Domain types
// We need bigdecimal for some fields
use bigdecimal::BigDecimal;
use std::str::FromStr;

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MockCompanyOverviewResponse {
    symbol: String,
    name: String,
    description: Option<String>,
    exchange: Option<String>,
    currency: Option<String>,
    country: Option<String>,
    sector: Option<String>,
    industry: Option<String>,
    market_capitalization: Option<String>,
    #[serde(rename = "EBITDA")]
    ebitda: Option<String>,
    #[serde(rename = "PERatio")]
    pe_ratio: Option<String>,
    #[serde(rename = "PEGRatio")]
    peg_ratio: Option<String>,
    book_value: Option<String>,
    dividend_per_share: Option<String>,
    dividend_yield: Option<String>,
    #[serde(rename = "EPS")]
    eps: Option<String>,
    #[serde(rename = "RevenuePerShareTTM")]
    revenue_per_share_ttm: Option<String>,
    profit_margin: Option<String>,
    #[serde(rename = "OperatingMarginTTM")]
    operating_margin_ttm: Option<String>,
    #[serde(rename = "ReturnOnAssetsTTM")]
    return_on_assets_ttm: Option<String>,
    #[serde(rename = "ReturnOnEquityTTM")]
    return_on_equity_ttm: Option<String>,
    #[serde(rename = "RevenueTTM")]
    revenue_ttm: Option<String>,
    #[serde(rename = "GrossProfitTTM")]
    gross_profit_ttm: Option<String>,
    #[serde(rename = "DilutedEPSTTM")]
    diluted_eps_ttm: Option<String>,
    #[serde(rename = "QuarterlyEarningsGrowthYOY")]
    quarterly_earnings_growth_yoy: Option<String>,
    #[serde(rename = "QuarterlyRevenueGrowthYOY")]
    quarterly_revenue_growth_yoy: Option<String>,
    analyst_target_price: Option<String>,
    #[serde(rename = "TrailingPE")]
    trailing_pe: Option<String>,
    #[serde(rename = "ForwardPE")]
    forward_pe: Option<String>,
    #[serde(rename = "PriceToSalesRatioTTM")]
    price_to_sales_ratio_ttm: Option<String>,
    price_to_book_ratio: Option<String>,
    #[serde(rename = "EVToRevenue")]
    ev_to_revenue: Option<String>,
    #[serde(rename = "EVToEBITDA")]
    ev_to_ebitda: Option<String>,
    beta: Option<String>,
    #[serde(rename = "52WeekHigh")]
    week_52_high: Option<String>,
    #[serde(rename = "52WeekLow")]
    week_52_low: Option<String>,
    #[serde(rename = "50DayMovingAverage")]
    day_50_moving_average: Option<String>,
    #[serde(rename = "200DayMovingAverage")]
    day_200_moving_average: Option<String>,
    shares_outstanding: Option<String>,
    shares_float: Option<String>,
    percent_insiders: Option<String>,
    percent_institutions: Option<String>,
    dividend_date: Option<String>,
    ex_dividend_date: Option<String>,
}

impl From<MockCompanyOverviewResponse> for CompanyOverview {
    fn from(val: MockCompanyOverviewResponse) -> CompanyOverview {
        CompanyOverview {
            symbol: val.symbol,
            name: val.name,
            description: val.description,
            exchange: val.exchange,
            currency: val.currency,
            country: val.country,
            sector: val.sector,
            industry: val.industry,
            market_capitalization: val.market_capitalization.and_then(|s| s.parse().ok()),
            ebitda: val.ebitda.and_then(|s| s.parse().ok()),
            pe_ratio: val.pe_ratio.and_then(|s| s.parse().ok()),
            peg_ratio: val.peg_ratio.and_then(|s| s.parse().ok()),
            book_value: val.book_value.and_then(|s| s.parse().ok()),
            dividend_per_share: val.dividend_per_share.and_then(|s| s.parse().ok()),
            dividend_yield: val.dividend_yield.and_then(|s| s.parse().ok()),
            eps: val.eps.and_then(|s| s.parse().ok()),
            revenue_per_share_ttm: val.revenue_per_share_ttm.and_then(|s| s.parse().ok()),
            profit_margin: val.profit_margin.and_then(|s| s.parse().ok()),
            operating_margin_ttm: val.operating_margin_ttm.and_then(|s| s.parse().ok()),
            return_on_assets_ttm: val.return_on_assets_ttm.and_then(|s| s.parse().ok()),
            return_on_equity_ttm: val.return_on_equity_ttm.and_then(|s| s.parse().ok()),
            revenue_ttm: val.revenue_ttm.and_then(|s| s.parse().ok()),
            gross_profit_ttm: val.gross_profit_ttm.and_then(|s| s.parse().ok()),
            diluted_eps_ttm: val.diluted_eps_ttm.and_then(|s| s.parse().ok()),
            quarterly_earnings_growth_yoy: val
                .quarterly_earnings_growth_yoy
                .and_then(|s| s.parse().ok()),
            quarterly_revenue_growth_yoy: val
                .quarterly_revenue_growth_yoy
                .and_then(|s| s.parse().ok()),
            analyst_target_price: val.analyst_target_price.and_then(|s| s.parse().ok()),
            trailing_pe: val.trailing_pe.and_then(|s| s.parse().ok()),
            forward_pe: val.forward_pe.and_then(|s| s.parse().ok()),
            price_to_sales_ratio_ttm: val.price_to_sales_ratio_ttm.and_then(|s| s.parse().ok()),
            price_to_book_ratio: val.price_to_book_ratio.and_then(|s| s.parse().ok()),
            ev_to_revenue: val.ev_to_revenue.and_then(|s| s.parse().ok()),
            ev_to_ebitda: val.ev_to_ebitda.and_then(|s| s.parse().ok()),
            beta: val.beta.and_then(|s| s.parse().ok()),
            week_52_high: val.week_52_high.and_then(|s| s.parse().ok()),
            week_52_low: val.week_52_low.and_then(|s| s.parse().ok()),
            day_50_moving_average: val.day_50_moving_average.and_then(|s| s.parse().ok()),
            day_200_moving_average: val.day_200_moving_average.and_then(|s| s.parse().ok()),
            shares_outstanding: val.shares_outstanding.and_then(|s| s.parse().ok()),
            shares_float: val.shares_float.and_then(|s| s.parse().ok()),
            percent_insiders: val.percent_insiders.and_then(|s| s.parse().ok()),
            percent_institutions: val.percent_institutions.and_then(|s| s.parse().ok()),
            dividend_date: val
                .dividend_date
                .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
            ex_dividend_date: val
                .ex_dividend_date
                .and_then(|s| chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok()),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomeStatementHelper {
    fiscal_date_ending: String,
    total_revenue: Option<String>,
    gross_profit: Option<String>,
    operating_income: Option<String>,
    net_income: Option<String>,
    // eps is not in income statement reports typically? Wait, it is if we check file.
    // The file has "netIncome", no "eps" in reports?
    // Let's check inome-statement-output.json again...
    // It has "totalRevenue", "grossProfit", "operatingIncome", "netIncome". No EPS.
    // EPS is usually in Earnings. But domain::IncomeStatement has eps.
}

impl From<IncomeStatementHelper> for IncomeStatement {
    fn from(val: IncomeStatementHelper) -> IncomeStatement {
        IncomeStatement {
            period_end_date: chrono::NaiveDate::parse_from_str(&val.fiscal_date_ending, "%Y-%m-%d")
                .unwrap_or_default(),
            revenue: val
                .total_revenue
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            gross_profit: val.gross_profit.and_then(|s| BigDecimal::from_str(&s).ok()),
            operating_income: val
                .operating_income
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            net_income: val.net_income.and_then(|s| BigDecimal::from_str(&s).ok()),
            eps: None, // Not in mock data
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BalanceSheetHelper {
    fiscal_date_ending: String,
    total_assets: Option<String>,
    total_liabilities: Option<String>,
    total_shareholder_equity: Option<String>,
    cash_and_cash_equivalents_at_carrying_value: Option<String>,
    short_term_investments: Option<String>,
    short_term_debt: Option<String>,
    long_term_debt: Option<String>,
    // net_debt not explicitly in file usually?
    common_stock_shares_outstanding: Option<String>,
}

impl From<BalanceSheetHelper> for BalanceSheet {
    fn from(val: BalanceSheetHelper) -> BalanceSheet {
        BalanceSheet {
            period_end_date: chrono::NaiveDate::parse_from_str(&val.fiscal_date_ending, "%Y-%m-%d")
                .unwrap_or_default(),
            total_assets: val.total_assets.and_then(|s| BigDecimal::from_str(&s).ok()),
            total_liabilities: val
                .total_liabilities
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            total_equity: val
                .total_shareholder_equity
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            cash_and_equivalents: val
                .cash_and_cash_equivalents_at_carrying_value
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            short_term_investments: val
                .short_term_investments
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            short_term_debt: val
                .short_term_debt
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            long_term_debt: val
                .long_term_debt
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            net_debt: None,
            common_stock_shares_outstanding: val
                .common_stock_shares_outstanding
                .and_then(|s| s.parse().ok()),
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct CashFlowHelper {
    fiscal_date_ending: String,
    operating_cashflow: Option<String>,
    capital_expenditures: Option<String>,
    // free_cash_flow usually calculated or field? "freeCashFlow"?
    // I'll check file content conceptually or just assume minimal
}

impl From<CashFlowHelper> for CashFlowStatement {
    fn from(val: CashFlowHelper) -> CashFlowStatement {
        CashFlowStatement {
            period_end_date: chrono::NaiveDate::parse_from_str(&val.fiscal_date_ending, "%Y-%m-%d")
                .unwrap_or_default(),
            operating_cash_flow: val
                .operating_cashflow
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            capital_expenditures: val
                .capital_expenditures
                .and_then(|s| BigDecimal::from_str(&s).ok()),
            free_cash_flow: None, // Calculate if needed: operating - capex
        }
    }
}

#[derive(Clone, Default)]
pub struct MockObjectStorage;

impl MockObjectStorage {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl ObjectStorage for MockObjectStorage {
    async fn put_object(
        &self,
        _key: &str,
        _data: Bytes,
        _content_type: &str,
    ) -> Result<(), AppError> {
        Ok(())
    }
    async fn get_object(&self, _key: &str) -> Result<Bytes, AppError> {
        Ok(Bytes::new())
    }
    async fn get_presigned_url(
        &self,
        _key: &str,
        _expires_in: Duration,
    ) -> Result<String, AppError> {
        Ok("http://localhost".to_string())
    }
    async fn delete_object(&self, _key: &str) -> Result<(), AppError> {
        Ok(())
    }
}
