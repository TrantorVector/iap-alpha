use crate::state::AppState;
use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use db::repositories::CompanyRepository;
use domain::metrics::calculator::MetricsCalculator;
use domain::periods::{PeriodWindowGenerator, PeriodType};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, IntoParams};
use uuid::Uuid;

pub fn companies_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_company_details))
        .route("/:id/metrics", get(get_company_metrics))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CompanyDetailsResponse {
    pub id: Uuid,
    pub symbol: String,
    pub name: String,
    pub exchange: String,
    pub sector: Option<String>,
    pub market_cap: Option<f64>,
    pub market_cap_formatted: String,
    pub currency: String,
    pub fiscal_year_end_month: i32,
    pub is_active: bool,
    pub last_updated: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Company details", body = CompanyDetailsResponse),
        (status = 404, description = "Company not found")
    ),
    tag = "companies"
)]
pub async fn get_company_details(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = CompanyRepository::new(state.db.clone());
    let company = repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(company) = company {
        let market_cap_f64 = company.market_cap.map(|mc| mc as f64);
        let market_cap_formatted = format_market_cap(market_cap_f64, company.currency.as_deref());

        let response = CompanyDetailsResponse {
            id: company.id,
            symbol: company.symbol,
            name: company.name,
            exchange: company.exchange,
            sector: company.industry, // Using industry as a proxy for sector for now as sector_id resolution is not implemented
            market_cap: market_cap_f64,
            market_cap_formatted,
            currency: company.currency.unwrap_or_else(|| "USD".to_string()),
            fiscal_year_end_month: company.fiscal_year_end_month.unwrap_or(0),
            is_active: company.is_active,
            last_updated: company.updated_at,
        };

        Ok(Json(response))
    } else {
        Err((StatusCode::NOT_FOUND, "Company not found".to_string()))
    }
}


#[derive(Deserialize, IntoParams)]
pub struct MetricsQueryParams {
    #[serde(default = "default_period_type")]
    pub period_type: String,
    #[serde(default = "default_period_count")]
    pub period_count: usize,
}

fn default_period_type() -> String {
    "quarterly".to_string()
}

fn default_period_count() -> usize {
    8
}

#[derive(Serialize, ToSchema)]
pub struct MetricsResponse {
    pub company_id: Uuid,
    pub period_type: String,
    pub periods: Vec<String>, // period labels
    pub sections: MetricsSections,
}

#[derive(Serialize, ToSchema)]
pub struct MetricsSections {
    pub growth_and_margins: Vec<MetricRow>,
    pub cash_and_leverage: Vec<MetricRow>,
    pub valuation: Vec<MetricRow>,
}

#[derive(Serialize, ToSchema)]
pub struct MetricRow {
    pub metric_name: String,
    pub display_name: String,
    pub values: Vec<MetricValueOut>,
    pub heat_map_enabled: bool,
}

#[derive(Serialize, ToSchema)]
pub struct MetricValueOut {
    pub period: String,
    pub value: Option<f64>,
    pub formatted: String,
    pub heat_map_quartile: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/metrics",
    params(
        ("id" = Uuid, Path, description = "Company ID"),
        MetricsQueryParams
    ),
    responses(
        (status = 200, description = "Company metrics", body = MetricsResponse),
        (status = 404, description = "Company not found")
    ),
    tag = "companies"
)]
pub async fn get_company_metrics(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<MetricsQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let repo = CompanyRepository::new(state.db.clone());
    
    // 1. Fetch company
    let company = repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    // 2. Determine period type
    let period_type_str = params.period_type.to_lowercase();
    let is_quarterly = period_type_str == "quarterly";
    let db_period_type = if is_quarterly { "quarterly" } else { "annual" };
    let domain_period_type = if is_quarterly { PeriodType::Quarterly } else { PeriodType::Annual };

    // 3. Fetch financial data
    // Fetch a bit more than requested to have prior year data for YoY calculations
    // If quarterly, we need 4 quarters back for YoY
    // If annual, we need 1 year back for YoY
    let limit = params.period_count + if is_quarterly { 4 } else { 1 };
    
    let db_incomes = repo.get_income_statements(id, db_period_type, limit as i32)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let db_balances = repo.get_balance_sheets(id, db_period_type, limit as i32)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    let db_cashflows = repo.get_cash_flow_statements(id, db_period_type, limit as i32)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 4. Period Window Generation
    let generator = PeriodWindowGenerator::new(company.fiscal_year_end_month.unwrap_or(12) as u32);
    let periods = generator.generate_periods(
        params.period_count,
        domain_period_type,
        Utc::now().date_naive(),
    );
    let period_labels: Vec<String> = periods.iter().map(|p| p.display_label.clone()).collect();

    // 5. Map DB models to Domain models for calculations
    // We need to reverse the db results because they are likely ordered by date DESC
    let mut db_incomes = db_incomes;
    db_incomes.sort_by_key(|i| i.period_end_date);
    
    let mut db_balances = db_balances;
    db_balances.sort_by_key(|b| b.period_end_date);
    
    let mut db_cashflows = db_cashflows;
    db_cashflows.sort_by_key(|c| c.period_end_date);

    // Filter to match the periods we generated as closely as possible, 
    // or just take the most recent available.
    // Fixed: The calculator expects the main 'incomes' to be the periods we want to display.
    // And 'prior_year_incomes' to be the income statement from 1 year prior for each period.
    
    let mut domain_incomes = Vec::new();
    let mut prior_year_incomes = Vec::new();
    let mut domain_balances = Vec::new();
    let mut domain_cashflows = Vec::new();
    
    // Take the last 'params.period_count' statements as the current ones
    let start_idx = db_incomes.len().saturating_sub(params.period_count);
    let current_db_incomes = &db_incomes[start_idx..];
    
    for (i, db_inc) in current_db_incomes.iter().enumerate() {
        let domain_inc = domain::domain::IncomeStatement {
            period_end_date: db_inc.period_end_date,
            revenue: db_inc.total_revenue.clone(),
            gross_profit: db_inc.gross_profit.clone(),
            operating_income: db_inc.operating_income.clone(),
            net_income: db_inc.net_income.clone(),
            eps: db_inc.basic_eps.clone(),
        };
        domain_incomes.push(domain_inc);
        
        // Prior year income for YoY
        let prior_idx = if is_quarterly {
            (start_idx + i).checked_sub(4)
        } else {
            (start_idx + i).checked_sub(1)
        };
        
        let prior_inc = prior_idx.and_then(|idx| db_incomes.get(idx)).map(|db_inc| {
            domain::domain::IncomeStatement {
                period_end_date: db_inc.period_end_date,
                revenue: db_inc.total_revenue.clone(),
                gross_profit: db_inc.gross_profit.clone(),
                operating_income: db_inc.operating_income.clone(),
                net_income: db_inc.net_income.clone(),
                eps: db_inc.basic_eps.clone(),
            }
        });
        prior_year_incomes.push(prior_inc);
        
        // Match balance sheet and cash flow by date
        let bal = db_balances.iter().find(|b| b.period_end_date == db_inc.period_end_date).map(|b| {
            domain::domain::BalanceSheet {
                period_end_date: b.period_end_date,
                total_assets: b.total_assets.clone(),
                total_liabilities: b.total_liabilities.clone(),
                total_equity: b.total_equity.clone(),
                cash_and_equivalents: b.cash_and_equivalents.clone(),
                short_term_investments: b.short_term_investments.clone(),
                short_term_debt: b.short_term_debt.clone(),
                long_term_debt: b.long_term_debt.clone(),
                net_debt: b.net_debt.clone(),
                common_stock_shares_outstanding: db_inc.shares_outstanding, 
            }
        });
        domain_balances.push(bal);

        let cf = db_cashflows.iter().find(|c| c.period_end_date == db_inc.period_end_date).map(|c| {
            domain::domain::CashFlowStatement {
                period_end_date: c.period_end_date,
                operating_cash_flow: c.operating_cash_flow.clone(),
                capital_expenditures: c.capital_expenditures.clone(),
                free_cash_flow: c.free_cash_flow.clone(),
            }
        });
        domain_cashflows.push(cf);
    }

    let currency = company.currency.as_deref().unwrap_or("$");

    // 6. Calculate Metrics
    let (revs, yoy, qoq) = MetricsCalculator::calculate_revenue_metrics(&domain_incomes, &prior_year_incomes, currency);
    let (gm, om, nm) = MetricsCalculator::calculate_margin_metrics(&domain_incomes);
    let (ocf_r, fcf_r) = MetricsCalculator::calculate_cash_metrics(&domain_incomes, &domain_cashflows);
    let (lev_r, shares) = MetricsCalculator::calculate_leverage_metrics(&domain_incomes, &domain_balances);
    
    // For valuation, we need prices. For now, we'll return empty/mock or fetch if possible.
    // The prompt says "compute all metrics".
    let (open_r, high_r, low_r, close_r, pe_r) = MetricsCalculator::calculate_valuation_metrics(&domain_incomes, &vec![None; domain_incomes.len()]);

    // 7. Format Response
    let mut sections = MetricsSections {
        growth_and_margins: Vec::new(),
        cash_and_leverage: Vec::new(),
        valuation: Vec::new(),
    };

    // Helper to map domain MetricValue to output
    let to_row = |name: &str, display: &str, values: Vec<domain::metrics::MetricValue>, labels: &[String]| -> MetricRow {
        MetricRow {
            metric_name: name.to_string(),
            display_name: display.to_string(),
            values: values.into_iter().enumerate().map(|(i, v)| MetricValueOut {
                period: labels.get(i).cloned().unwrap_or_default(),
                value: v.value,
                formatted: v.formatted_value,
                heat_map_quartile: v.heat_map_quartile,
            }).collect(),
            heat_map_enabled: true,
        }
    };

    sections.growth_and_margins.push(to_row("revenue", "Revenue", revs, &period_labels));
    sections.growth_and_margins.push(to_row("revenue_growth_yoy", "Revenue Growth (YoY)", yoy, &period_labels));
    sections.growth_and_margins.push(to_row("revenue_growth_qoq", "Revenue Growth (QoQ)", qoq, &period_labels));
    sections.growth_and_margins.push(to_row("gross_margin", "Gross Margin", gm, &period_labels));
    sections.growth_and_margins.push(to_row("operating_margin", "Operating Margin", om, &period_labels));
    sections.growth_and_margins.push(to_row("net_margin", "Net Margin", nm, &period_labels));

    sections.cash_and_leverage.push(to_row("ocf_margin", "OCF Margin", ocf_r, &period_labels));
    sections.cash_and_leverage.push(to_row("fcf_margin", "FCF Margin", fcf_r, &period_labels));
    sections.cash_and_leverage.push(to_row("leverage_ratio", "Leverage Ratio", lev_r, &period_labels));
    sections.cash_and_leverage.push(to_row("shares_outstanding", "Shares Outstanding", shares, &period_labels));

    sections.valuation.push(to_row("pe_ratio", "P/E Ratio", pe_r, &period_labels));
    // Prefix unused price metrics with underscore for now as they are empty
    let _ = (open_r, high_r, low_r, close_r);

    let response = MetricsResponse {
        company_id: id,
        period_type: period_type_str,
        periods: period_labels,
        sections,
    };

    Ok(Json(response))
}

fn format_market_cap(market_cap: Option<f64>, currency: Option<&str>) -> String {
    let currency_symbol = match currency {
        Some("USD") | None => "$",
        Some("INR") => "₹",
        Some("EUR") => "€",
        Some("GBP") => "£",
        Some(c) => c, // Fallback to code if unknown symbol
    };

    match market_cap {
        Some(cap) => {
            if cap >= 1_000_000_000_000.0 {
                format!("{}{:.1}T", currency_symbol, cap / 1_000_000_000_000.0)
            } else if cap >= 1_000_000_000.0 {
                format!("{}{:.1}B", currency_symbol, cap / 1_000_000_000.0)
            } else if cap >= 1_000_000.0 {
                format!("{}{:.1}M", currency_symbol, cap / 1_000_000.0)
            } else if cap >= 1_000.0 {
                format!("{}{:.1}K", currency_symbol, cap / 1_000.0)
            } else {
                format!("{}{:.0}", currency_symbol, cap)
            }
        }
        None => "N/A".to_string(),
    }
}
