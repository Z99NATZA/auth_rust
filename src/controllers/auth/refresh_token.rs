use axum::{Json, extract::State, http::StatusCode, response::{IntoResponse, Response}};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{EncodingKey, Header as JwtHeader};
use chrono::{Utc, Duration};
use uuid::Uuid;
use std::sync::Arc;
use cookie::time::Duration as CookieDuration;
use crate::{app::{error::AppError, result::AppResult, state::AppState}, controllers::auth::{login::{Claims, LoginResponse}, utils::{generate_refresh_token, hash_refresh_token}}, utils::env::env_i64};

pub async fn refresh(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> AppResult<Response> {
    let refresh_plain = jar.get("refresh_token")
        .ok_or(AppError::Unauthorized)?
        .value()
        .to_string();

    let hash = hash_refresh_token(&refresh_plain, &state.refresh_secret)?;

    // พยายามหา refresh token ที่ยังใช้ได้
    let rec_opt = sqlx::query!(
        r#"
            SELECT rt.id, rt.user_id, u.username, u.role, u.token_version
            FROM refresh_tokens rt
            JOIN users u ON u.id = rt.user_id
            WHERE rt.token_hash = $1
                AND rt.revoked_at IS NULL
                AND rt.expires_at > now()
        "#,
        hash
    )
    .fetch_optional(&state.db)
    .await?;

    let rec = match rec_opt {
        Some(r) => r,
        None => {
            // เช็คว่าเป็น "reuse" ไหม (ถูก revoke ไปแล้ว)
            // ใส่ type ให้ชัดเป็น bool เพื่อตัด Option ออก
            let reused: bool = sqlx::query_scalar!(
                r#"
                    SELECT EXISTS(
                    SELECT 1 FROM refresh_tokens
                    WHERE token_hash = $1 AND revoked_at IS NOT NULL
                    ) as "exists!: bool"
                "#,
                hash
            )
            .fetch_one(&state.db)
            .await?;

            if reused {
                // TODO: handle reuse (เช่น revoke ทั้ง user / log เหตุการณ์)
            }

            return Err(AppError::Unauthorized);
        }
    };

    // เพิกถอน refresh เดิมทันที (rotate)
    sqlx::query!("UPDATE refresh_tokens SET revoked_at = now() WHERE id = $1", rec.id)
        .execute(&state.db)
        .await?;

    // ออก access token ใหม่
    let now = Utc::now();

    // อ่านจาก ENV, ไม่มีก็ 15 นาที
    let ttl_min = env_i64("ACCESS_TTL_MIN", 15)?;

    // กันค่าพิสดารเล็กน้อย (เช่น ไม่ให้ติดลบ/ยาวเกิน)
    let ttl_min = ttl_min.clamp(1, 120);
    
    let exp = now + chrono::Duration::minutes(ttl_min);

    let claims = Claims {
        sub: rec.user_id,
        username: rec.username,
        role: rec.role,
        iat: now.timestamp() as usize,
        exp: exp.timestamp() as usize,
        iss: state.jwt_issuer.clone(),
        aud: state.jwt_audience.clone(),
        jti: Uuid::new_v4().to_string(),
        token_version: rec.token_version,
    };

    let access_token = jsonwebtoken::encode(
        &JwtHeader::default(),
        &claims,
        &EncodingKey::from_secret(&state.jwt_secret),
    )?;

    // ออก refresh ใหม่
    let new_plain = generate_refresh_token()?;
    let new_hash  = hash_refresh_token(&new_plain, &state.refresh_secret)?;

    // หมดอายุ 30 วัน
    let new_exp = Utc::now() + Duration::days(30);

    sqlx::query!(
        r#"INSERT INTO refresh_tokens (user_id, token_hash, expires_at)
           VALUES ($1, $2, $3)"#,
        rec.user_id, new_hash, new_exp
    )
    .execute(&state.db)
    .await?;

    // เซ็ตคุกกี้ใหม่
    let mut refresh_cookie = cookie::Cookie::new("refresh_token", new_plain);
    refresh_cookie.set_http_only(true);
    refresh_cookie.set_secure(false); // dev = false
    // refresh_cookie.set_secure(true);
    refresh_cookie.set_same_site(cookie::SameSite::Lax);
    refresh_cookie.set_path("/auth");
    refresh_cookie.set_max_age(CookieDuration::days(30));

    // เตรียม response
    let jar = jar.add(refresh_cookie);

    let body = LoginResponse {
        access_token: access_token,
        token_type: "Bearer".into(),
        expires_in: (exp - now).num_seconds(),
    };

    // คืน (CookieJar, Response)
    Ok((jar, (StatusCode::OK, Json(body))).into_response())
}
