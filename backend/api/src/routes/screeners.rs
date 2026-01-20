use crate::auth::jwt::Claims;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use db::models::screener::Screener;
use db::repositories::screener_repository::{CreateScreener, ScreenerRepository, UpdateScreener};
use domain::services::screener_service::{FilterCriteria, ScreenerResult, ScreenerService};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

pub fn screeners_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_screeners).post(create_screener))
        .route(
            "/:id",
            get(get_screener)
                .put(update_screener)
                .delete(delete_screener),
        )
        .route("/:id/run", post(run_screener))
}

// DTOs for API Documentation (ToSchema support)

#[derive(Serialize, ToSchema)]
pub struct ScreenerResponse {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    #[schema(value_type = Object)]
    pub filter_criteria: serde_json::Value,
    #[schema(value_type = Option<Object>)]
    pub sort_config: Option<serde_json::Value>,
    #[schema(value_type = Option<Object>)]
    pub display_columns: Option<serde_json::Value>,
    pub display_order: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Screener> for ScreenerResponse {
    fn from(s: Screener) -> Self {
        Self {
            id: s.id,
            title: s.title,
            description: s.description,
            filter_criteria: s.filter_criteria,
            sort_config: s.sort_config,
            display_columns: s.display_columns,
            display_order: s.display_order,
            created_at: s.created_at,
            updated_at: s.updated_at,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct CreateScreenerRequest {
    pub title: String,
    pub description: Option<String>,
    #[schema(value_type = Object)]
    pub filter_criteria: serde_json::Value,
    #[schema(value_type = Option<Object>)]
    pub sort_config: Option<serde_json::Value>,
    #[schema(value_type = Option<Object>)]
    pub display_columns: Option<serde_json::Value>,
}

impl From<CreateScreenerRequest> for CreateScreener {
    fn from(val: CreateScreenerRequest) -> Self {
        CreateScreener {
            title: val.title,
            description: val.description,
            filter_criteria: val.filter_criteria,
            sort_config: val.sort_config,
            display_columns: val.display_columns,
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateScreenerRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub filter_criteria: Option<serde_json::Value>,
    #[schema(value_type = Option<Object>)]
    pub sort_config: Option<serde_json::Value>,
    #[schema(value_type = Option<Object>)]
    pub display_columns: Option<serde_json::Value>,
}

impl From<UpdateScreenerRequest> for UpdateScreener {
    fn from(val: UpdateScreenerRequest) -> Self {
        UpdateScreener {
            title: val.title,
            description: val.description,
            filter_criteria: val.filter_criteria,
            sort_config: val.sort_config,
            display_columns: val.display_columns,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct ScreenerResultsResponse {
    pub screener_id: Uuid,
    pub executed_at: DateTime<Utc>,
    pub total_results: i32,
    pub results: Vec<ScreenerResult>,
}

#[derive(Deserialize, ToSchema, IntoParams)]
pub struct RunScreenerRequest {
    pub override_criteria: Option<FilterCriteria>,
}

// Handlers

#[utoipa::path(
    get,
    path = "/api/v1/screeners",
    responses(
        (status = 200, description = "List user screeners", body = Vec<ScreenerResponse>),
    ),
    tag = "screeners"
)]
pub async fn list_screeners(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = ScreenerRepository::new(state.db.clone());
    let screeners = repo
        .list_by_user(user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response: Vec<ScreenerResponse> = screeners.into_iter().map(Into::into).collect();

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/api/v1/screeners",
    request_body = CreateScreenerRequest,
    responses(
        (status = 201, description = "Screener created", body = ScreenerResponse),
        (status = 400, description = "Invalid input")
    ),
    tag = "screeners"
)]
pub async fn create_screener(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<CreateScreenerRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = ScreenerRepository::new(state.db.clone());
    let screener = repo
        .create(user_id, payload.into())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ScreenerResponse::from(screener))))
}

#[utoipa::path(
    get,
    path = "/api/v1/screeners/{id}",
    params(
        ("id" = Uuid, Path, description = "Screener ID")
    ),
    responses(
        (status = 200, description = "Screener details", body = ScreenerResponse),
        (status = 404, description = "Screener not found")
    ),
    tag = "screeners"
)]
pub async fn get_screener(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = ScreenerRepository::new(state.db.clone());
    let screener = repo
        .find_by_id(id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Screener not found".to_string()))?;

    Ok(Json(ScreenerResponse::from(screener)))
}

#[utoipa::path(
    put,
    path = "/api/v1/screeners/{id}",
    params(
        ("id" = Uuid, Path, description = "Screener ID")
    ),
    request_body = UpdateScreenerRequest,
    responses(
        (status = 200, description = "Screener updated", body = ScreenerResponse),
        (status = 404, description = "Screener not found")
    ),
    tag = "screeners"
)]
pub async fn update_screener(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateScreenerRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    // Check existence first
    let repo = ScreenerRepository::new(state.db.clone());

    let existing = repo
        .find_by_id(id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_none() {
        return Err((StatusCode::NOT_FOUND, "Screener not found".to_string()));
    }

    let updated = repo
        .update(id, user_id, payload.into())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ScreenerResponse::from(updated)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/screeners/{id}",
    params(
        ("id" = Uuid, Path, description = "Screener ID")
    ),
    responses(
        (status = 204, description = "Screener deleted"),
        (status = 404, description = "Screener not found")
    ),
    tag = "screeners"
)]
pub async fn delete_screener(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = ScreenerRepository::new(state.db.clone());
    let deleted = repo
        .delete(id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if deleted {
        Ok((StatusCode::NO_CONTENT, ()))
    } else {
        Err((StatusCode::NOT_FOUND, "Screener not found".to_string()))
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/screeners/{id}/run",
    params(
        ("id" = Uuid, Path, description = "Screener ID")
    ),
    request_body = Option<RunScreenerRequest>,
    responses(
        (status = 200, description = "Screener results", body = ScreenerResultsResponse),
        (status = 404, description = "Screener not found")
    ),
    tag = "screeners"
)]
pub async fn run_screener(
    State(state): State<AppState>,
    claims: Claims,
    Path(id): Path<Uuid>,
    // Optional request body
    payload: Option<Json<RunScreenerRequest>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid user ID".to_string()))?;

    let repo = ScreenerRepository::new(state.db.clone());

    // 1. Fetch screener definition
    let screener = repo
        .find_by_id(id, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Screener not found".to_string()))?;

    // 2. Determine criteria to use
    let criteria: FilterCriteria = if let Some(Json(req)) = payload {
        if let Some(override_criteria) = req.override_criteria {
            override_criteria
        } else {
            // Parse from screener.filter_criteria
            serde_json::from_value(screener.filter_criteria).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid stored criteria: {}", e),
                )
            })?
        }
    } else {
        // Parse from screener.filter_criteria
        serde_json::from_value(screener.filter_criteria).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Invalid stored criteria: {}", e),
            )
        })?
    };

    // 3. Execute screener
    let service = ScreenerService::new(state.db.clone());
    let results = service
        .execute(criteria)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let response = ScreenerResultsResponse {
        screener_id: id,
        executed_at: Utc::now(),
        total_results: results.len() as i32,
        results,
    };

    Ok(Json(response))
}
