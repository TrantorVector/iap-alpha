use crate::state::AppState;
use axum::{extract::State, response::IntoResponse, Json};

pub async fn login(State(_state): State<AppState>) -> impl IntoResponse {
    Json("Login placeholder")
}

pub async fn refresh_token(State(_state): State<AppState>) -> impl IntoResponse {
    Json("Refresh token placeholder")
}
