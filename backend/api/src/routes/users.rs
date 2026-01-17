use crate::state::AppState;
use axum::{routing::get, Router};

pub fn user_router() -> Router<AppState> {
    Router::new().route("/", get(get_current_user))
}

async fn get_current_user() -> &'static str {
    "Get current user placeholder"
}
