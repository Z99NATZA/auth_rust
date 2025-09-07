use std::{net::IpAddr, sync::Arc};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::{HeaderMap, StatusCode, header}, response::{IntoResponse, Response}};
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, Header, EncodingKey};
use sqlx::types::ipnet::IpNet;
use uuid::Uuid;
use cookie::time::OffsetDateTime;
use crate::{app::{error::AppError, result::AppResult, state::AppState}, controllers::auth::utils::{generate_refresh_token, hash_refresh_token}};

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
    pub access_token: String,
    pub token_type: String, // "Bearer"
    pub expires_in: i64, // วินาที
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Response> {
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
        iss: state.jwt_issuer.clone(),
        aud: state.jwt_audience.clone(),
        jti: Uuid::new_v4().to_string(),
        token_version: user.token_version,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret),
    )?;

    // ออก refresh token
    let refresh_plain = generate_refresh_token()?;
    let refresh_hash = hash_refresh_token(&refresh_plain, &state.refresh_secret)?;
    
    // หมดอายุ 30 วัน
    let refresh_exp = Utc::now() + Duration::days(30);

    // refresh_exp = chrono::DateTime<Utc> ---> แปลงเป็น OffsetDateTime
    let expires = OffsetDateTime::from_unix_timestamp(refresh_exp.timestamp())
        .map_err(|e| AppError::InternalError(format!("valid timestamp: {:?}", e).into()))?;

    // browser / app / library
    let user_agent: Option<String> = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // IP
    let ip: Option<IpAddr> = Some("192.168.1.10".parse().unwrap());

    // แปลงเป็น IpNet
    let ip: Option<IpNet> = ip.map(IpNet::from);

    sqlx::query!(
        r#"
            INSERT INTO refresh_tokens (user_id, token_hash, user_agent, ip, expires_at)
            VALUES ($1, $2, $3, $4, $5)
        "#,
        user.id,
        refresh_hash,
        user_agent,
        ip,
        refresh_exp
    )
    .execute(&state.db)
    .await?;

    // สร้าง refresh cookie
    // ตั้งค่าแบบ universal (ไม่พึ่ง builder API)
    let mut refresh_cookie = Cookie::new("refresh_token", refresh_plain);
    refresh_cookie.set_http_only(true);
    refresh_cookie.set_secure(true);
    refresh_cookie.set_same_site(SameSite::Lax); // หรือ Strict
    refresh_cookie.set_path("/auth");
    refresh_cookie.set_expires(expires);

    // เตรียม response
    let res = LoginResponse { 
        access_token: token.clone(), 
        token_type: "Bearer".into(), 
        expires_in: (exp - now).num_seconds() 
    };

    let response = (
        StatusCode::OK,
        [(header::SET_COOKIE, refresh_cookie.to_string())],
        Json(res),
    ).into_response();

    Ok(response)
}