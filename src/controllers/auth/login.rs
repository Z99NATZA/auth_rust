use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};

use crate::app::{error::AppError, result::AppResult, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: uuid::Uuid,
    pub username: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    access_token: String,
    token_type: String, // "Bearer"
    expires_in: i64, // วินาที
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<(StatusCode, Json<LoginResponse>)> {
    let user = sqlx::query!(
        r#"SELECT id, username, password_hash, role, is_active FROM users WHERE username = $1"#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    if !user.is_active.unwrap_or(false) { 
        return Err(AppError::Unauthorized);
    }
   
    let parsed_hash = PasswordHash::new(&user.password_hash)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::Unauthorized)?;

    let now = Utc::now();
    let exp = now + Duration::hours(2);
    let claims = Claims {
        sub: user.id,
        username: user.username,
        role: user.role.clone().unwrap_or_else(|| "user".to_string()),
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    )?;

    let res = LoginResponse {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: (exp - now).num_seconds(),
    };
    
    Ok((StatusCode::OK, Json(res)))
}