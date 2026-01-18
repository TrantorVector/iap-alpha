use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use bytes::Bytes;
use chrono::Datelike;
use chrono::{DateTime, NaiveDate, Utc};
use db::repositories::{CompanyRepository, CreateDocumentParams, DocumentRepository};
use db::PgPool;
use domain::metrics::calculator::{MetricsCalculator, ValuationMetrics};
use domain::periods::{PeriodType, PeriodWindowGenerator};
use multer::Multipart;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx;
use std::time::Duration;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub fn companies_router() -> Router<AppState> {
    Router::new()
        .route("/:id", get(get_company_details))
        .route("/:id/metrics", get(get_company_metrics))
        .route(
            "/:id/documents",
            get(get_company_documents).post(upload_company_document),
        )
        .route(
            "/:id/documents/:doc_id/download",
            get(get_document_download_url),
        )
        .route("/:id/verdict", get(get_verdict).put(update_verdict))
        .route("/:id/verdict/history", get(get_verdict_history))
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
    let domain_period_type = if is_quarterly {
        PeriodType::Quarterly
    } else {
        PeriodType::Annual
    };

    // 3. Fetch financial data
    // Fetch a bit more than requested to have prior year data for YoY calculations
    // If quarterly, we need 4 quarters back for YoY
    // If annual, we need 1 year back for YoY
    let limit = params.period_count + if is_quarterly { 4 } else { 1 };

    let db_incomes = repo
        .get_income_statements(id, db_period_type, limit as i32)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let db_balances = repo
        .get_balance_sheets(id, db_period_type, limit as i32)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let db_cashflows = repo
        .get_cash_flow_statements(id, db_period_type, limit as i32)
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
        let bal = db_balances
            .iter()
            .find(|b| b.period_end_date == db_inc.period_end_date)
            .map(|b| domain::domain::BalanceSheet {
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
            });
        domain_balances.push(bal);

        let cf = db_cashflows
            .iter()
            .find(|c| c.period_end_date == db_inc.period_end_date)
            .map(|c| domain::domain::CashFlowStatement {
                period_end_date: c.period_end_date,
                operating_cash_flow: c.operating_cash_flow.clone(),
                capital_expenditures: c.capital_expenditures.clone(),
                free_cash_flow: c.free_cash_flow.clone(),
            });
        domain_cashflows.push(cf);
    }

    let currency = company.currency.as_deref().unwrap_or("$");

    // 6. Calculate Metrics
    let (revs, yoy, qoq) = MetricsCalculator::calculate_revenue_metrics(
        &domain_incomes,
        &prior_year_incomes,
        currency,
    );
    let (gm, om, nm) = MetricsCalculator::calculate_margin_metrics(&domain_incomes);
    let (ocf_r, fcf_r) =
        MetricsCalculator::calculate_cash_metrics(&domain_incomes, &domain_cashflows);
    let (lev_r, shares) =
        MetricsCalculator::calculate_leverage_metrics(&domain_incomes, &domain_balances);

    // For valuation, we need prices. For now, we'll return empty/mock or fetch if possible.
    // The prompt says "compute all metrics".
    let ValuationMetrics {
        open_ratios: open_r,
        high_ratios: high_r,
        low_ratios: low_r,
        close_ratios: close_r,
        pe_ratios: pe_r,
    } = MetricsCalculator::calculate_valuation_metrics(
        &domain_incomes,
        &vec![None; domain_incomes.len()],
    );

    // 7. Format Response
    let mut sections = MetricsSections {
        growth_and_margins: Vec::new(),
        cash_and_leverage: Vec::new(),
        valuation: Vec::new(),
    };

    // Helper to map domain MetricValue to output
    let to_row = |name: &str,
                  display: &str,
                  values: Vec<domain::metrics::MetricValue>,
                  labels: &[String]|
     -> MetricRow {
        MetricRow {
            metric_name: name.to_string(),
            display_name: display.to_string(),
            values: values
                .into_iter()
                .enumerate()
                .map(|(i, v)| MetricValueOut {
                    period: labels.get(i).cloned().unwrap_or_default(),
                    value: v.value,
                    formatted: v.formatted_value,
                    heat_map_quartile: v.heat_map_quartile,
                })
                .collect(),
            heat_map_enabled: true,
        }
    };

    sections
        .growth_and_margins
        .push(to_row("revenue", "Revenue", revs, &period_labels));
    sections.growth_and_margins.push(to_row(
        "revenue_growth_yoy",
        "Revenue Growth (YoY)",
        yoy,
        &period_labels,
    ));
    sections.growth_and_margins.push(to_row(
        "revenue_growth_qoq",
        "Revenue Growth (QoQ)",
        qoq,
        &period_labels,
    ));
    sections
        .growth_and_margins
        .push(to_row("gross_margin", "Gross Margin", gm, &period_labels));
    sections.growth_and_margins.push(to_row(
        "operating_margin",
        "Operating Margin",
        om,
        &period_labels,
    ));
    sections
        .growth_and_margins
        .push(to_row("net_margin", "Net Margin", nm, &period_labels));

    sections
        .cash_and_leverage
        .push(to_row("ocf_margin", "OCF Margin", ocf_r, &period_labels));
    sections
        .cash_and_leverage
        .push(to_row("fcf_margin", "FCF Margin", fcf_r, &period_labels));
    sections.cash_and_leverage.push(to_row(
        "leverage_ratio",
        "Leverage Ratio",
        lev_r,
        &period_labels,
    ));
    sections.cash_and_leverage.push(to_row(
        "shares_outstanding",
        "Shares Outstanding",
        shares,
        &period_labels,
    ));

    sections
        .valuation
        .push(to_row("pe_ratio", "P/E Ratio", pe_r, &period_labels));
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

#[derive(Deserialize, IntoParams)]
pub struct DocumentsQueryParams {
    pub document_type: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct DocumentsResponse {
    pub documents: Vec<DocumentOut>,
    pub freshness: FreshnessMetadata,
}

#[derive(Serialize, ToSchema)]
pub struct DocumentOut {
    pub id: Uuid,
    pub document_type: String,
    pub period_end_date: Option<NaiveDate>,
    pub fiscal_year: i32,
    pub fiscal_quarter: Option<i32>,
    pub title: String,
    pub source_url: Option<String>,
    pub storage_key: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub available: bool,
}

#[derive(Serialize, ToSchema)]
pub struct FreshnessMetadata {
    pub last_refreshed_at: Option<DateTime<Utc>>,
    pub is_stale: bool,
    pub refresh_requested: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/documents",
    params(
        ("id" = Uuid, Path, description = "Company ID"),
        DocumentsQueryParams
    ),
    responses(
        (status = 200, description = "Company documents", body = DocumentsResponse),
        (status = 404, description = "Company not found")
    ),
    tag = "companies"
)]
pub async fn get_company_documents(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<DocumentsQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let company_repo = CompanyRepository::new(state.db.clone());
    let doc_repo = DocumentRepository::new(state.db.clone());

    // 1. Fetch company for freshness and existence
    let company = company_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    // 2. Fetch documents
    let docs = doc_repo
        .find_by_company_id(id, params.document_type)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Determine freshness
    let last_refreshed_at = Some(company.updated_at);
    let is_stale = Utc::now()
        .signed_duration_since(company.updated_at)
        .num_hours()
        > 24;
    let refresh_requested = false; // Could check background_jobs table for pending refreshes

    if is_stale {
        // Enqueue background refresh (placeholder)
        tracing::info!("Company {} data is stale, would enqueue refresh", id);
    }

    // 4. Map to response (grouped by type and sorted by date via DB)
    let documents = docs
        .into_iter()
        .map(|d| {
            let available = d.is_available();
            DocumentOut {
                id: d.id,
                document_type: d.document_type,
                period_end_date: d.period_end_date,
                fiscal_year: d
                    .fiscal_year
                    .unwrap_or_else(|| d.period_end_date.map(|dt| dt.year()).unwrap_or(0)),
                fiscal_quarter: d.fiscal_quarter,
                title: d.title,
                source_url: d.source_url,
                storage_key: d.storage_key,
                file_size: d.file_size,
                mime_type: d.mime_type,
                available,
            }
        })
        .collect();

    let response = DocumentsResponse {
        documents,
        freshness: FreshnessMetadata {
            last_refreshed_at,
            is_stale,
            refresh_requested,
        },
    };

    Ok(Json(response))
}

#[derive(Serialize, ToSchema)]
pub struct DownloadResponse {
    pub download_url: String,
    pub expires_in: i64,
    pub filename: String,
    pub content_type: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/documents/{doc_id}/download",
    params(
        ("id" = Uuid, Path, description = "Company ID"),
        ("doc_id" = Uuid, Path, description = "Document ID")
    ),
    responses(
        (status = 200, description = "Document download URL", body = DownloadResponse),
        (status = 404, description = "Document not found"),
        (status = 400, description = "Document not available"),
        (status = 403, description = "Access denied")
    ),
    tag = "companies"
)]
pub async fn get_document_download_url(
    State(state): State<AppState>,
    Path((id, doc_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let doc_repo = DocumentRepository::new(state.db.clone());

    // 1. Fetch document
    let doc = doc_repo
        .find_by_id(doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;

    // 2. Verify document belongs to company
    if doc.company_id != id {
        return Err((
            StatusCode::FORBIDDEN,
            "Document does not belong to this company".to_string(),
        ));
    }

    // 3. Verify document has storage_key
    let storage_key = doc.storage_key.ok_or((
        StatusCode::BAD_REQUEST,
        "Document is not yet available for download".to_string(),
    ))?;

    // 4. Generate presigned URL
    let expires_in = Duration::from_secs(15 * 60);
    let download_url = state
        .storage
        .get_presigned_url(&storage_key, expires_in)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 5. Response
    let filename = format!(
        "{}_{}.pdf",
        doc.document_type,
        doc.period_end_date
            .map(|d| d.to_string())
            .unwrap_or_else(|| "unknown".into())
    );

    let response = DownloadResponse {
        download_url,
        expires_in: 900,
        filename,
        content_type: doc
            .mime_type
            .unwrap_or_else(|| "application/pdf".to_string()),
    };

    Ok(Json(response))
}

#[derive(Serialize, ToSchema)]
pub struct DocumentUploadResponse {
    pub id: Uuid,
    pub document_type: String,
    pub period_end_date: Option<NaiveDate>,
    pub title: String,
    pub storage_key: String,
    pub file_size: i64,
    pub mime_type: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/companies/{id}/documents",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    responses(
        (status = 201, description = "Document uploaded successfully", body = DocumentUploadResponse),
        (status = 400, description = "Invalid file or parameters"),
        (status = 404, description = "Company not found"),
        (status = 413, description = "File too large")
    ),
    tag = "companies"
)]
pub async fn upload_company_document(
    State(state): State<AppState>,
    Path(company_id): Path<Uuid>,
    request: Request<Body>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let company_repo = CompanyRepository::new(state.db.clone());
    let doc_repo = DocumentRepository::new(state.db.clone());

    // 1. Verify company exists
    company_repo
        .find_by_id(company_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    // 2. Parse multipart form
    let boundary = request
        .headers()
        .get("content-type")
        .and_then(|ct| ct.to_str().ok())
        .and_then(|ct| multer::parse_boundary(ct).ok())
        .ok_or((StatusCode::BAD_REQUEST, "Invalid content-type".to_string()))?;

    // Use the body stream directly with multer
    let stream = request.into_body().into_data_stream();
    let mut multipart = Multipart::new(stream, boundary);

    // 3. Extract form fields
    let mut file_data: Option<Bytes> = None;
    let mut file_name: Option<String> = None;
    let mut document_type: Option<String> = None;
    let mut period_end_date: Option<NaiveDate> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Multipart error: {}", e)))?
    {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                file_data = Some(field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file: {}", e),
                    )
                })?);
            }
            "document_type" => {
                document_type = Some(field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read document_type: {}", e),
                    )
                })?);
            }
            "period_end_date" => {
                let date_str = field.text().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read period_end_date: {}", e),
                    )
                })?;
                if !date_str.is_empty() {
                    period_end_date = Some(
                        NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|_| {
                            (
                                StatusCode::BAD_REQUEST,
                                "Invalid date format (use YYYY-MM-DD)".to_string(),
                            )
                        })?,
                    );
                }
            }
            _ => {}
        }
    }

    // 4. Validate required fields
    let file_data = file_data.ok_or((StatusCode::BAD_REQUEST, "File is required".to_string()))?;
    let file_name =
        file_name.ok_or((StatusCode::BAD_REQUEST, "File name is required".to_string()))?;
    let document_type = document_type.ok_or((
        StatusCode::BAD_REQUEST,
        "document_type is required".to_string(),
    ))?;

    // 5. Validate file size (already checked during read, but double-check)
    let file_size = file_data.len() as i64;
    if file_size > 52_428_800 {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "File too large (max 50MB)".to_string(),
        ));
    }

    // 6. Validate file type based on extension
    let extension = file_name.rsplit('.').next().unwrap_or("").to_lowercase();

    let mime_type = match extension.as_str() {
        "pdf" => "application/pdf",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        _ => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid file type. Allowed: PDF, PPT, PPTX, DOC, DOCX".to_string(),
            ))
        }
    };

    // 7. Generate storage key
    let file_uuid = Uuid::new_v4();
    let storage_key = format!("documents/{}/{}/{}", company_id, file_uuid, file_name);

    // 8. Upload to S3
    state
        .storage
        .put_object(&storage_key, file_data, mime_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Storage error: {}", e),
            )
        })?;

    // 9. Determine fiscal year and quarter from period_end_date if provided
    let fiscal_year = period_end_date.map(|d| d.year());
    let fiscal_quarter = None; // Could calculate this based on company fiscal year end month

    // 10. Create document record
    let title = format!("{} - {}", document_type, file_name);
    let document = doc_repo
        .create(CreateDocumentParams {
            company_id,
            document_type: document_type.clone(),
            period_end_date,
            fiscal_year,
            fiscal_quarter,
            title,
            storage_key: storage_key.clone(),
            source_url: None, // source_url (not applicable for uploads)
            file_size,
            mime_type: mime_type.to_string(),
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 11. Return response
    let response = DocumentUploadResponse {
        id: document.id,
        document_type: document.document_type,
        period_end_date: document.period_end_date,
        title: document.title,
        storage_key: document.storage_key.unwrap_or_default(),
        file_size: document.file_size.unwrap_or(0),
        mime_type: document.mime_type.unwrap_or_default(),
    };

    Ok((StatusCode::CREATED, Json(response)))
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

#[derive(Serialize, ToSchema)]
pub struct VerdictResponse {
    pub verdict_id: Option<Uuid>,
    pub company_id: Uuid,
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    pub guidance_summary: Option<String>,
    pub lock_version: i32,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub linked_reports: Vec<LinkedReport>,
}

#[derive(Serialize, ToSchema)]
pub struct LinkedReport {
    pub report_id: Uuid,
    pub filename: String,
    pub uploaded_at: DateTime<Utc>,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/verdict",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Current verdict for the company", body = VerdictResponse),
        (status = 404, description = "Company not found")
    ),
    tag = "companies"
)]
pub async fn get_verdict(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use db::repositories::VerdictRepository;

    let company_repo = CompanyRepository::new(state.db.clone());
    let verdict_repo = VerdictRepository::new(state.db.clone());

    // 1. Verify company exists
    company_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    // 2. For now, we need a user_id. In production this would come from JWT auth.
    // For testing purposes, we'll fetch the first user or use a test user.
    // TODO: Replace with actual user ID from authentication middleware
    let test_user_id = get_test_user_id(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // 3. Fetch current verdict for company and user
    let verdict_opt = verdict_repo
        .find_by_company(id, test_user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = match verdict_opt {
        Some(verdict) => {
            // Fetch linked analysis reports
            let linked_reports = fetch_linked_reports(&state.db, verdict.id)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Parse JSON arrays for strengths and weaknesses
            let strengths = parse_json_to_strings(verdict.strengths);
            let weaknesses = parse_json_to_strings(verdict.weaknesses);

            VerdictResponse {
                verdict_id: Some(verdict.id),
                company_id: verdict.company_id,
                final_verdict: verdict.final_verdict,
                summary_text: verdict.summary_text,
                strengths,
                weaknesses,
                guidance_summary: verdict.guidance_summary,
                lock_version: verdict.lock_version,
                created_at: Some(verdict.created_at),
                updated_at: Some(verdict.updated_at),
                linked_reports,
            }
        }
        None => {
            // Return empty response for companies with no verdict
            VerdictResponse {
                verdict_id: None,
                company_id: id,
                final_verdict: None,
                summary_text: None,
                strengths: Vec::new(),
                weaknesses: Vec::new(),
                guidance_summary: None,
                lock_version: 0,
                created_at: None,
                updated_at: None,
                linked_reports: Vec::new(),
            }
        }
    };

    Ok(Json(response))
}

// Helper function to get test user ID
// TODO: Remove this when authentication middleware is fully integrated
async fn get_test_user_id(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    let result: (Uuid,) =
        sqlx::query_as("SELECT id FROM users WHERE username = 'testuser' LIMIT 1")
            .fetch_optional(pool)
            .await?
            .unwrap_or_else(|| {
                // Return a default UUID if no test user exists
                // This should never happen in a properly seeded database
                (Uuid::nil(),)
            });

    Ok(result.0)
}

// Helper function to fetch linked analysis reports
async fn fetch_linked_reports(
    pool: &PgPool,
    verdict_id: Uuid,
) -> Result<Vec<LinkedReport>, sqlx::Error> {
    let reports = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
        r#"
        SELECT id, filename, uploaded_at
        FROM analysis_reports
        WHERE verdict_id = $1
        ORDER BY uploaded_at DESC
        "#,
    )
    .bind(verdict_id)
    .fetch_all(pool)
    .await?;

    Ok(reports
        .into_iter()
        .map(|(id, filename, uploaded_at)| LinkedReport {
            report_id: id,
            filename,
            uploaded_at,
        })
        .collect())
}

// Helper function to parse JSON value to Vec<String>
fn parse_json_to_strings(json_value: Option<serde_json::Value>) -> Vec<String> {
    json_value
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

#[derive(Deserialize, ToSchema)]
pub struct VerdictUpdateRequest {
    pub lock_version: i32,
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,
    #[serde(default)]
    pub strengths: Vec<String>,
    #[serde(default)]
    pub weaknesses: Vec<String>,
    pub guidance_summary: Option<String>,
    #[serde(default)]
    pub linked_report_ids: Vec<Uuid>,
}

#[utoipa::path(
    put,
    path = "/api/v1/companies/{id}/verdict",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    request_body = VerdictUpdateRequest,
    responses(
        (status = 200, description = "Verdict updated successfully", body = VerdictResponse),
        (status = 404, description = "Company not found"),
        (status = 409, description = "Conflict - Optimistic lock version mismatch")
    ),
    tag = "companies"
)]
pub async fn update_verdict(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<VerdictUpdateRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    use db::repositories::{VerdictCreate, VerdictRepository, VerdictUpdate};
    use db::DbError;

    let company_repo = CompanyRepository::new(state.db.clone());
    let verdict_repo = VerdictRepository::new(state.db.clone());

    // 1. Verify company exists
    match company_repo.find_by_id(id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Company not found"})),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ))
        }
    }

    // 2. Validate and map final_verdict
    let mapped_verdict = if let Some(ref v) = payload.final_verdict {
        match v.as_str() {
            "INVEST" => Some("invest".to_string()),
            "PASS" => Some("pass".to_string()),
            "WATCHLIST" => Some("watchlist".to_string()),
            "NO_THESIS" => None,
            _ => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({"error": "Invalid final_verdict value"})),
                ))
            }
        }
    } else {
        None
    };

    // 3. Get User ID (Test User for now)
    let user_id = match get_test_user_id(&state.db).await {
        Ok(uid) => uid,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ))
        }
    };

    // 4. Check if verdict exists
    let existing_verdict = verdict_repo
        .find_by_company(id, user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let verdict = match existing_verdict {
        Some(current) => {
            // Create History Snapshot of the CURRENT state (before update)
            match verdict_repo.create_history_snapshot(current.id).await {
                Ok(history) => {
                    // Link reports currently linked to this verdict to the new history snapshot
                    if let Err(e) = verdict_repo
                        .link_reports_to_history(current.id, history.id)
                        .await
                    {
                        tracing::error!("Failed to link reports to history snapshot: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to create history snapshot: {}", e);
                    // Proceed with update even if snapshot fails?
                    // Ideally we should fail, but for now log and proceed to avoid blocking user.
                }
            }

            // Update existing
            let update_dto = VerdictUpdate {
                final_verdict: mapped_verdict,
                summary_text: payload.summary_text,
                strengths: Some(json!(payload.strengths)),
                weaknesses: Some(json!(payload.weaknesses)),
                guidance_summary: payload.guidance_summary,
            };

            match verdict_repo
                .update_with_lock(current.id, update_dto, payload.lock_version)
                .await
            {
                Ok(v) => v,
                Err(DbError::OptimisticLockError(_)) => {
                    // Fetch fresh state for 409 details
                    let fresh_state = verdict_repo.find_by_id(current.id).await.ok().flatten();

                    let current_response = if let Some(fresh) = fresh_state {
                        let v_strengths = parse_json_to_strings(fresh.strengths);
                        let v_weaknesses = parse_json_to_strings(fresh.weaknesses);
                        let linked = fetch_linked_reports(&state.db, fresh.id)
                            .await
                            .unwrap_or_default();

                        Some(VerdictResponse {
                            verdict_id: Some(fresh.id),
                            company_id: fresh.company_id,
                            final_verdict: fresh.final_verdict.map(|s| s.to_uppercase()), // Map back to UPPERCASE for response?
                            summary_text: fresh.summary_text,
                            strengths: v_strengths,
                            weaknesses: v_weaknesses,
                            guidance_summary: fresh.guidance_summary,
                            lock_version: fresh.lock_version,
                            created_at: Some(fresh.created_at),
                            updated_at: Some(fresh.updated_at),
                            linked_reports: linked,
                        })
                    } else {
                        None
                    };

                    return Err((
                        StatusCode::CONFLICT,
                        Json(json!({
                            "error": {
                                "code": "CONFLICT",
                                "details": {
                                    "message": "Resource modified by another request",
                                    "current_version": current.lock_version,
                                    "current_state": current_response
                                }
                            }
                        })),
                    ));
                }
                Err(e) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": e.to_string()})),
                    ))
                }
            }
        }
        None => {
            let create_dto = VerdictCreate {
                final_verdict: mapped_verdict,
                summary_text: payload.summary_text,
                strengths: Some(json!(payload.strengths)),
                weaknesses: Some(json!(payload.weaknesses)),
                guidance_summary: payload.guidance_summary,
            };

            verdict_repo
                .create(id, user_id, create_dto)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": e.to_string()})),
                    )
                })?
        }
    };

    // 5. Update Linked Reports
    if let Err(e) = update_linked_reports(&state.db, verdict.id, &payload.linked_report_ids).await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to update linked reports: {}", e)})),
        ));
    }

    // 6. Build Response
    let linked_reports = fetch_linked_reports(&state.db, verdict.id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
        })?;

    let strengths = parse_json_to_strings(verdict.strengths);
    let weaknesses = parse_json_to_strings(verdict.weaknesses);

    let response = VerdictResponse {
        verdict_id: Some(verdict.id),
        company_id: verdict.company_id,
        final_verdict: verdict.final_verdict.map(|s| s.to_uppercase()),
        summary_text: verdict.summary_text,
        strengths,
        weaknesses,
        guidance_summary: verdict.guidance_summary,
        lock_version: verdict.lock_version,
        created_at: Some(verdict.created_at),
        updated_at: Some(verdict.updated_at),
        linked_reports,
    };

    Ok(Json(response))
}

async fn update_linked_reports(
    pool: &PgPool,
    verdict_id: Uuid,
    report_ids: &[Uuid],
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Clear existing links
    sqlx::query("UPDATE analysis_reports SET verdict_id = NULL WHERE verdict_id = $1")
        .bind(verdict_id)
        .execute(&mut *tx)
        .await?;

    if !report_ids.is_empty() {
        // Link new list
        sqlx::query("UPDATE analysis_reports SET verdict_id = $1 WHERE id = ANY($2)")
            .bind(verdict_id)
            .bind(report_ids)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[derive(Serialize, ToSchema)]
pub struct VerdictHistoryResponse {
    pub company_id: Uuid,
    pub history: Vec<VerdictHistoryEntry>,
}

#[derive(Serialize, ToSchema)]
pub struct VerdictHistoryEntry {
    pub history_id: Uuid,
    pub version: i32,
    pub final_verdict: Option<String>,
    pub summary_text: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub linked_report: Option<LinkedReport>,
}

#[utoipa::path(
    get,
    path = "/api/v1/companies/{id}/verdict/history",
    params(
        ("id" = Uuid, Path, description = "Company ID")
    ),
    responses(
        (status = 200, description = "Verdict history", body = VerdictHistoryResponse),
        (status = 404, description = "Company not found")
    ),
    tag = "companies"
)]
pub async fn get_verdict_history(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    use db::repositories::VerdictRepository;

    let company_repo = CompanyRepository::new(state.db.clone());
    let verdict_repo = VerdictRepository::new(state.db.clone());

    // 1. Verify company exists
    company_repo
        .find_by_id(id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Company not found".to_string()))?;

    // 2. Fetch verdict to get ID (needed for history)
    // We need user_id again.
    let test_user_id = get_test_user_id(&state.db)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let verdict = verdict_repo
        .find_by_company(id, test_user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let history_entries = if let Some(verdict) = verdict {
        // 3. Fetch history
        let mut history = verdict_repo
            .get_history(verdict.id)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

        // 4. Order descending (repo is ASC) and limit to 50
        history.reverse();
        history.truncate(50);

        // 5. Map to Response Entry
        let mut entries = Vec::new();
        for h in history {
            // Fetch linked report for this history version
            // Assuming 1 report per history version based on schema usage or prompt implication
            let linked_report = sqlx::query_as::<_, (Uuid, String, DateTime<Utc>)>(
                r#"
                SELECT id, filename, uploaded_at
                FROM analysis_reports
                WHERE verdict_history_id = $1
                LIMIT 1
                "#,
            )
            .bind(h.id)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
            .map(|(id, filename, uploaded_at)| LinkedReport {
                report_id: id,
                filename,
                uploaded_at,
            });

            entries.push(VerdictHistoryEntry {
                history_id: h.id,
                version: h.version,
                final_verdict: h.final_verdict,
                summary_text: h.summary_text,
                recorded_at: h.recorded_at,
                linked_report,
            });
        }
        entries
    } else {
        Vec::new()
    };

    Ok(Json(VerdictHistoryResponse {
        company_id: id,
        history: history_entries,
    }))
}
