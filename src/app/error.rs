#![allow(dead_code)]

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use thiserror::Error;
use serde_json::json;
use sqlx::Error as SqlxError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Env variable error: {0}")]
    EnvVarError(#[from] std::env::VarError),

    #[error("Dotenv loading error: {0}")]
    DotenvError(#[from] dotenv::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Internal server error")]
    InternalError(String),

    #[error("Not Found")]
    NotFound,

    #[error("JSON decode error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Bad request")]
    BadRequest(String),

    #[error("SQLx error: {0}")]
    SqlxError(#[from] SqlxError),

    #[error("Argon2 error: {0}")]
    Argon2Error(#[from] argon2::password_hash::Error),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error(transparent)]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("Base64 decode error: {0}")]
    Base64DecodeError(#[from] base64::DecodeError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message): (StatusCode, String) = match self {
            AppError::EnvVarError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::DotenvError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::InternalError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::IoError(e) => (StatusCode::BAD_GATEWAY, e.to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::JsonError(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::SqlxError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Argon2Error(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string()),
            AppError::Base64DecodeError(e) => (StatusCode::BAD_REQUEST, e.to_string()),
            AppError::JwtError(e) => {
                (StatusCode::UNAUTHORIZED, e.to_string())
            },
        };

        let body = Json(json!({
            "error": message
        }));

        (status, body).into_response()
    }
}