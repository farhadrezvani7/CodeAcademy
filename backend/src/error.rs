use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// API error. Internal details are logged, never returned to the client.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("not found: {0}")]
    NotFound(&'static str),
    #[error("validation: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    /// No valid session.
    #[error("unauthorized")]
    Unauthorized,
    /// Wrong credentials (message shown to the user).
    #[error("unauthenticated: {0}")]
    Unauthenticated(String),
    #[error("too many requests: {0}")]
    TooManyRequests(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound(what) => (StatusCode::NOT_FOUND, "not_found", format!("{what} پیدا نشد.")),
            AppError::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, "validation_error", msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "conflict", msg.clone()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "ابتدا وارد حساب کاربری‌ات شو.".into()),
            AppError::Unauthenticated(msg) => (StatusCode::UNAUTHORIZED, "invalid_credentials", msg.clone()),
            AppError::TooManyRequests(msg) => (StatusCode::TOO_MANY_REQUESTS, "too_many_requests", msg.clone()),
            AppError::Database(e) => {
                tracing::error!(error = %e, "database error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "خطای داخلی سرور رخ داد. کمی بعد دوباره تلاش کن.".into())
            }
            AppError::Internal(e) => {
                tracing::error!(error = %e, "internal error");
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "خطای داخلی سرور رخ داد. کمی بعد دوباره تلاش کن.".into())
            }
        };
        (status, Json(json!({ "error": { "code": code, "message": message } }))).into_response()
    }
}
