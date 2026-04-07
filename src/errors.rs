//! Custom error types and error handling for the Derivative backend.
//!
//! Provides consistent error responses across all API endpoints.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use thiserror::Error;
use tracing::error;

/// Application-wide error type
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authentication required")]
    Unauthorized,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Access forbidden")]
    Forbidden,

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("File upload error: {0}")]
    FileUpload(String),

    #[error("Invalid file type: {0}")]
    InvalidFileType(String),

    #[error("File too large")]
    FileTooLarge,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

/// Error code for consistent API responses
#[derive(Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    DatabaseError,
    ValidationError,
    Unauthorized,
    InvalidCredentials,
    TokenExpired,
    InvalidToken,
    Forbidden,
    NotFound,
    Conflict,
    BadRequest,
    FileUploadError,
    InvalidFileType,
    FileTooLarge,
    RateLimitExceeded,
    InternalServerError,
}

/// Structured error response body
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<String>>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::Database(e) => {
                error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::DatabaseError,
                    "An internal database error occurred".to_string(),
                    None,
                )
            }
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::ValidationError,
                msg.clone(),
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::Unauthorized,
                "Authentication required".to_string(),
                None,
            ),
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::InvalidCredentials,
                "Invalid email or password".to_string(),
                None,
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::TokenExpired,
                "Token has expired".to_string(),
                None,
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                ErrorCode::InvalidToken,
                "Invalid or malformed token".to_string(),
                None,
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                ErrorCode::Forbidden,
                "You do not have permission to access this resource".to_string(),
                None,
            ),
            AppError::NotFound(resource) => (
                StatusCode::NOT_FOUND,
                ErrorCode::NotFound,
                format!("{} not found", resource),
                None,
            ),
            AppError::Conflict(msg) => {
                (StatusCode::CONFLICT, ErrorCode::Conflict, msg.clone(), None)
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::BadRequest,
                msg.clone(),
                None,
            ),
            AppError::FileUpload(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::FileUploadError,
                msg.clone(),
                None,
            ),
            AppError::InvalidFileType(msg) => (
                StatusCode::BAD_REQUEST,
                ErrorCode::InvalidFileType,
                msg.clone(),
                None,
            ),
            AppError::FileTooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ErrorCode::FileTooLarge,
                "File size exceeds the maximum allowed limit".to_string(),
                None,
            ),
            AppError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorCode::RateLimitExceeded,
                "Too many requests, please try again later".to_string(),
                None,
            ),
            AppError::Internal(e) => {
                error!("Internal error: {:?}", e);
                // In development mode, include more details
                let message = if std::env::var("RUST_LOG")
                    .map(|v| v.contains("debug"))
                    .unwrap_or(false)
                {
                    format!("Internal error: {}", e)
                } else {
                    "An internal error occurred".to_string()
                };
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorCode::InternalServerError,
                    message,
                    None,
                )
            }
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                code,
                message,
                details,
            },
        });

        (status, body).into_response()
    }
}

/// Result type alias for handlers
pub type AppResult<T> = Result<T, AppError>;

/// Validation error helper
pub fn validation_error(field: &str, message: &str) -> AppError {
    AppError::Validation(format!("{}: {}", field, message))
}

/// Convert validator errors to AppError
impl From<validator::ValidationErrors> for AppError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!(
                        "{}: {}",
                        field,
                        error
                            .message
                            .as_ref()
                            .map(|m| m.to_string())
                            .unwrap_or_else(|| "invalid".to_string())
                    )
                })
            })
            .collect();

        AppError::Validation(messages.join(", "))
    }
}
