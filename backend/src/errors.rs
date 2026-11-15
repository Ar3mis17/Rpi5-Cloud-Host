//! Error types

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use shared::ErrorResponse;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("Authentication error: {0}")]
    Auth(String),
    #[error("Authorization error: {0}")]
    Forbidden(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("Argon2 error: {0}")]
    Argon2(#[from] argon2::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Upload error: {0}")]
    Upload(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "Database error occurred"),
            AppError::Migration(_) => (StatusCode::INTERNAL_SERVER_ERROR, "MIGRATION_ERROR", "Migration error"),
            AppError::Auth(msg) => (StatusCode::UNAUTHORIZED, "AUTH_ERROR", msg.as_str()),
            AppError::Forbidden(msg) => (StatusCode::FORBIDDEN, "FORBIDDEN", msg.as_str()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.as_str()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.as_str()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.as_str()),
            AppError::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMITED", "Too many requests"),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", msg.as_str()),
            AppError::Jwt(_) => (StatusCode::UNAUTHORIZED, "JWT_ERROR", "Invalid token"),
            AppError::Argon2(_) => (StatusCode::INTERNAL_SERVER_ERROR, "HASH_ERROR", "Password hashing error"),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", "File system error"),
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "CONFIG_ERROR", msg.as_str()),
            AppError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "STORAGE_ERROR", msg.as_str()),
            AppError::Upload(msg) => (StatusCode::BAD_REQUEST, "UPLOAD_ERROR", msg.as_str()),
            AppError::Serialization(_) => (StatusCode::INTERNAL_SERVER_ERROR, "SERIALIZATION_ERROR", "Serialization error"),
            AppError::Uuid(_) => (StatusCode::BAD_REQUEST, "UUID_ERROR", "Invalid UUID"),
            AppError::Anyhow(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal error"),
        };

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message: message.to_string(),
            details: None,
        });

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
