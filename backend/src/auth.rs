//! Request identity from the session cookie.

use crate::{error::AppError, repo::accounts, services::auth as auth_service, state::AppState};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, HeaderMap},
};

#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: i32,
    /// Raw session token (needed to keep this session on password change).
    pub token: String,
}

pub fn session_token(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == auth_service::SESSION_COOKIE)
        .map(|(_, v)| v.to_string())
        .filter(|v| v.len() == 64 && v.chars().all(|c| c.is_ascii_hexdigit()))
}

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = session_token(&parts.headers).ok_or(AppError::Unauthorized)?;
        let id = accounts::session_user(&state.pool, &auth_service::hash_token(&token), state.clock.now())
            .await?
            .ok_or(AppError::Unauthorized)?;
        Ok(CurrentUser { id, token })
    }
}
