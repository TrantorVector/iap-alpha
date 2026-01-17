use crate::state::AppState;
use axum::{routing::get, Router};

pub fn companies_router() -> Router<AppState> {
    Router::new().route("/", get(list_companies))
}

async fn list_companies() -> &'static str {
    "List companies placeholder"
}
