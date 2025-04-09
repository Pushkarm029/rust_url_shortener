use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("URL not found: {0}")]
    NotFound(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    status: u16,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InvalidUrl(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
        };

        tracing::error!(%status, error_message = %error_message, "Request failed");

        let error_response = ErrorResponse {
            error: error_message,
            status: status.as_u16(),
        };

        (status, Json(error_response)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
