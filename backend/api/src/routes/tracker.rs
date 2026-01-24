use crate::auth::jwt::Claims;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use chrono::{DateTime, NaiveDate, Utc};
use db::repositories::tracker_repository::{
    Pagination, TrackerRepository, VerdictFilters as TrackerFilters,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub fn tracker_router() -> Router<AppState> {
    Router::new()
        .route("/verdicts", get(list_verdicts))
        .route("/summary", get(get_summary))
}

// DTOs for API Documentation

#[derive(Serialize, ToSchema)]
pub struct TrackerSummaryResponse {
    pub total_analyzed: i64,
    pub invest_count: i64,
    pub pass_count: i64,
    pub watchlist_count: i64,
    pub no_thesis_count: i64,
    pub recent_activity: Vec<RecentActivityOut>,
}

#[derive(Serialize, ToSchema)]
pub struct RecentActivityOut {
    pub company_id: Uuid,
    pub symbol: String,
    pub company_name: String,
    pub verdict: String,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct VerdictListResponse {
    pub items: Vec<TrackerItemOut>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Serialize, ToSchema)]
pub struct TrackerItemOut {
    pub company_id: Uuid,
    pub symbol: String,
    pub company_name: String,
    pub exchange: String,
    pub sector: Option<String>,
    pub verdict: String,
    pub verdict_date: DateTime<Utc>,
    pub summary_text: String,
    pub version: i32,
}

#[derive(Deserialize, IntoParams)]
pub struct TrackerQueryParams {
    pub verdict_type: Option<Vec<String>>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub sector: Option<Vec<String>>,
    pub search: Option<String>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

// Handlers

#[utoipa::path(
    get,
    path = "/api/v1/tracker/verdicts",
    params(TrackerQueryParams),
    responses(
        (status = 200, description = "List verdicts", body = VerdictListResponse),
    ),
    tag = "tracker"
)]
pub async fn list_verdicts(
    State(state): State<AppState>,
    claims: Claims,
    Query(params): Query<TrackerQueryParams>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = TrackerRepository::new(state.db.clone());
    
    let filters = TrackerFilters {
        verdict_type: params.verdict_type,
        date_from: params.date_from,
        date_to: params.date_to,
        sector: params.sector,
        search: params.search,
    };

    let pagination = Pagination {
        page: params.page.unwrap_or(1),
        per_page: params.per_page.unwrap_or(20),
    };

    let result = repo
        .list_verdicts(user_id, filters, pagination)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = VerdictListResponse {
        items: result.items.into_iter().map(|i| TrackerItemOut {
            company_id: i.company_id,
            symbol: i.symbol,
            company_name: i.company_name,
            exchange: i.exchange,
            sector: i.sector,
            verdict: i.verdict,
            verdict_date: i.verdict_date,
            summary_text: i.summary_text,
            version: i.version,
        }).collect(),
        total: result.total,
        page: result.page,
        per_page: result.per_page,
    };

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/v1/tracker/summary",
    responses(
        (status = 200, description = "Tracker summary", body = TrackerSummaryResponse),
    ),
    tag = "tracker"
)]
pub async fn get_summary(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = TrackerRepository::new(state.db.clone());
    let summary = repo
        .get_summary(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = TrackerSummaryResponse {
        total_analyzed: summary.total_analyzed,
        invest_count: summary.invest_count,
        pass_count: summary.pass_count,
        watchlist_count: summary.watchlist_count,
        no_thesis_count: summary.no_thesis_count,
        recent_activity: summary.recent_activity.into_iter().map(|a| RecentActivityOut {
            company_id: a.company_id,
            symbol: a.symbol,
            company_name: a.company_name,
            verdict: a.verdict,
            recorded_at: a.recorded_at,
        }).collect(),
    };

    Ok(Json(response))
}
