use crate::state::AppState;
use axum::{routing::get, Router};

pub fn screeners_router() -> Router<AppState> {
    Router::new().route("/", get(list_screeners))
}

async fn list_screeners() -> &'static str {
    "List screeners placeholder"
}
