#![allow(dead_code)]

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use thiserror::Error;
use sqlx::Error as SqlxError;
use tracing::error;

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

    #[error("HMAC key error")]
    HmacKeyError(#[from] hmac::digest::InvalidLength),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // กำหนด (status, code, message) ที่ "ปลอดภัยต่อการส่งกลับ client"
        let (status, code, message) = match &self {
            // 4xx แก้ได้
            AppError::BadRequest(msg) =>
                (StatusCode::BAD_REQUEST, "bad_request", msg.clone()),
            AppError::Unauthorized =>
                (StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized".into()),
            AppError::Forbidden =>
                (StatusCode::FORBIDDEN, "forbidden", "Forbidden".into()),
            AppError::NotFound =>
                (StatusCode::NOT_FOUND, "not_found", "Not found".into()),
            AppError::JsonError(_) =>
                (StatusCode::BAD_REQUEST, "invalid_json", "Invalid JSON".into()),
            AppError::Base64DecodeError(_) =>
                (StatusCode::BAD_REQUEST, "invalid_base64", "Invalid base64".into()),
            AppError::JwtError(_) =>
                (StatusCode::UNAUTHORIZED, "invalid_token", "Invalid or expired token".into()),

            // กรณี SQLx: แยก RowNotFound ---> 404
            AppError::SqlxError(e) if matches!(e, SqlxError::RowNotFound) =>
                (StatusCode::NOT_FOUND, "not_found", "Not found".into()),

            // ที่เหลือถือเป็น internal ทั้งหมด
            AppError::EnvVarError(_)
            | AppError::DotenvError(_)
            | AppError::IoError(_)
            | AppError::SqlxError(_)
            | AppError::Argon2Error(_)
            | AppError::HmacKeyError(_)
            | AppError::InternalError(_) =>
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", "Internal server error".into()),
        };

        // log รายละเอียดจริง ๆ ฝั่งเซิร์ฟเวอร์
        if status.is_server_error() {
            error!(error = ?self, "internal error");
        }

        let body = Json(json!({
            "error": { "code": code, "message": message }
        }));

        (status, body).into_response()
    }
}