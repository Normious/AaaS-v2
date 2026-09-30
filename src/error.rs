use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Invalid input: {0}")]
    Validation(String),

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Account locked until {0}")]
    AccountLocked(i64),

    #[error("Email not verified")]
    EmailNotVerified,

    #[error("2FA required")]
    TwoFactorRequired,

    #[error("Rate limit exceeded")]
    RateLimited { retry_after_seconds: u64 },

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Validation(m) => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".into()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            AppError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            AppError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::AccountLocked(_) => (StatusCode::LOCKED, "Account locked".into()),
            AppError::EmailNotVerified => (StatusCode::FORBIDDEN, "Email not verified".into()),
            AppError::TwoFactorRequired => (StatusCode::UNAUTHORIZED, "2FA code required".into()),
            AppError::RateLimited { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".into(),
            ),
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
            AppError::Internal(m) => {
                tracing::error!(error = %m, "Internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".into(),
                )
            }
        };

        let mut body = json!({
            "success": false,
            "error": message,
        });

        if let AppError::RateLimited {
            retry_after_seconds,
        } = self
        {
            body["retry_after_seconds"] = json!(retry_after_seconds);
        }

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
