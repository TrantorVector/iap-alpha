use axum::{
    extract::{FromRequestParts, State},
    http::{header, Request},
    middleware::Next,
    response::Response,
};
use axum::http::request::Parts;

use crate::state::AppState;
use crate::auth::jwt::Claims;
use domain::error::AppError;

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let token = extract_token(&req)?;
    let claims = state.jwt_service.validate_access_token(token)?;

    req.extensions_mut().insert(claims);
    Ok(next.run(req).await)
}

pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req.headers().get(header::AUTHORIZATION);
    
    if let Some(auth_val) = auth_header {
        let auth_str = auth_val
            .to_str()
            .map_err(|_| AppError::AuthError("Invalid Authorization header".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::AuthError("Invalid token scheme".to_string()));
        }

        let token = &auth_str[7..];
        let claims = state.jwt_service.validate_access_token(token)?;
        req.extensions_mut().insert(claims);
    }

    Ok(next.run(req).await)
}

fn extract_token<B>(req: &Request<B>) -> Result<&str, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or_else(|| AppError::AuthError("Missing Authorization header".to_string()))?
        .to_str()
        .map_err(|_| AppError::AuthError("Invalid Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AppError::AuthError("Invalid token scheme".to_string()));
    }

    Ok(&auth_header[7..])
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = crate::error::ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &AppState) -> Result<Self, Self::Rejection> {
        let claims = parts
            .extensions
            .get::<Claims>()
            .ok_or_else(|| {
                AppError::AuthError("Missing request claims".to_string())
            })
            .map_err(crate::error::ApiError)?;

        Ok(claims.clone())
    }
}
