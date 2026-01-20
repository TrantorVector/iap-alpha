use axum::{
    routing::{get, post},
    Router,
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::state::AppState;

pub mod auth;
pub mod companies;
pub mod health;
pub mod screeners;
pub mod users;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::refresh_token,
        auth::logout,
        companies::get_company_details,
        companies::get_company_metrics,
        companies::get_company_documents,
        companies::upload_company_document,
        companies::get_document_download_url,
        companies::get_verdict,
        companies::update_verdict,
        screeners::list_screeners,
        screeners::create_screener,
        screeners::get_screener,
        screeners::update_screener,
        screeners::delete_screener,
        screeners::run_screener,
        // More paths added as we implement them
    ),
    components(schemas(
        health::HealthResponse,
        auth::LoginRequest,
        auth::LoginResponse,
        auth::UserInfo,
        auth::RefreshRequest,
        auth::RefreshResponse,
        auth::LogoutRequest,
        companies::CompanyDetailsResponse,
        companies::MetricsResponse,
        companies::MetricsSections,
        companies::MetricRow,
        companies::MetricValueOut,
        companies::DocumentsResponse,
        companies::DocumentOut,
        companies::FreshnessMetadata,
        companies::DocumentUploadResponse,
        companies::DownloadResponse,
        companies::VerdictResponse,
        companies::VerdictUpdateRequest,
        companies::LinkedReport,
        domain::periods::FiscalPeriod,
        domain::periods::PeriodType,
        screeners::ScreenerResponse,
        screeners::CreateScreenerRequest,
        screeners::UpdateScreenerRequest,
        screeners::ScreenerResultsResponse,
        screeners::RunScreenerRequest,
        domain::services::screener_service::ScreenerResult,
        domain::services::screener_service::FilterCriteria,
    )),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "auth", description = "Authentication endpoints"),
        (name = "companies", description = "Company data endpoints"),
        (name = "screeners", description = "Screener endpoints"),
        (name = "verdicts", description = "Verdict endpoints"),
    )
)]
pub struct ApiDoc;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Public routes (no auth)
        .route("/api/health", get(health::health_check))
        .route("/api/v1/auth/login", post(auth::login))
        .route("/api/v1/auth/refresh", post(auth::refresh_token))
        // Protected routes (require auth)
        .nest("/api/v1", protected_routes(state.clone()))
        // Add middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive()) // Tighten for production
        .with_state(state)
}

fn protected_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/logout", post(auth::logout))
        .nest("/companies", companies::companies_router())
        .nest("/screeners", screeners::screeners_router())
        .nest("/users/me", users::user_router())
        .layer(axum::middleware::from_fn_with_state(
            state,
            crate::middleware::auth::auth_middleware,
        ))
}
