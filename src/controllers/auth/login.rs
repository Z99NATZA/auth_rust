use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use uuid::Uuid;

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
    pub iat: usize,
    pub exp: usize,
    pub iss: String,
    pub aud: String,
    pub jti: String,
    pub token_version: i32,
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
        r#"
            SELECT 
                id, 
                username, 
                password_hash, 
                role,
                is_active,
                token_version 
            FROM users WHERE username = $1
        "#,
        payload.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::NotFound)?;

    if !user.is_active { 
        return Err(AppError::Unauthorized);
    }
   
    let parsed_hash = PasswordHash::new(&user.password_hash)?;

    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            // เพิ่ม failed_attempts เมื่อพลาด
            // (ไม่ critical ถ้าอัปเดตพลาดก็ไม่ต้อง fail ทั้งคำขอ)
            let _ = sqlx::query!(
                "UPDATE users
                 SET failed_login_attempts = failed_login_attempts + 1,
                     locked_until = CASE WHEN failed_login_attempts + 1 >= 5
                                         THEN now() + interval '15 minutes'
                                         ELSE locked_until END
                 WHERE id = $1",
                user.id
            )
            .execute(&state.db);
            AppError::Unauthorized
        })?;

    // ผ่านแล้ว รีเซ็ตตัวนับ + อัปเดต last_login_at
    let _ = sqlx::query!(
        "UPDATE users
         SET failed_login_attempts = 0,
             locked_until = NULL,
             last_login_at = now()
         WHERE id = $1",
        user.id
    )
    .execute(&state.db)
    .await;

    let now = Utc::now();
    let exp = now + Duration::hours(2);

    let claims = Claims {
        sub: user.id,
        username: user.username,
        role: user.role,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        iss: state.jwt_issuer.clone(),           // ✅ เติมค่า
        aud: state.jwt_audience.clone(),         // ✅ เติมค่า
        jti: Uuid::new_v4().to_string(),         // ✅ สุ่มใหม่ทุกครั้ง
        token_version: user.token_version,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret),
    )?;

    let res = LoginResponse {
        access_token: token,
        token_type: "Bearer".into(),
        expires_in: (exp - now).num_seconds(),
    };
    
    Ok((StatusCode::OK, Json(res)))
}