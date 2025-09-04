use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::TypedHeader;
use axum_extra::headers::{Authorization, authorization::Bearer};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use std::sync::Arc;
use uuid::Uuid;

use crate::app::state::AppState;
use crate::app::error::AppError;
use crate::controllers::auth::login::Claims;

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: Uuid,
    pub username: String,
    pub role: String,
}

impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // 1) ดึง Bearer token
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized)?;

        // 2) ตั้งค่า Validation ให้ตรวจ exp/iss/aud/leeway
        let mut v = Validation::new(Algorithm::HS256);
        v.validate_exp = true;
        v.leeway = 30;
        v.set_issuer(&[&state.jwt_issuer]);
        v.set_audience(&[state.jwt_audience.clone()]);

        // 3) decode + verify
        let data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(&state.jwt_secret), // Vec<u8> ---> &[u8]
            &v,
        )?;

        let claims = data.claims;

        // 4) (ออปชัน) เช็ค jti blocklist ถ้ามีระบบเก็บ
        // if state.is_blocklisted(&claims.jti).await? {
        //     return Err(AppError::Unauthorized);
        // }

        // 5) โหลดผู้ใช้จาก DB เพื่อตรวจ token_version / is_active / password_changed_at
        let user = sqlx::query!(
            r#"
            SELECT
            id, username, role, is_active, token_version,
            password_changed_at as "password_changed_at: chrono::DateTime<chrono::Utc>"
            FROM users
            WHERE id = $1
            "#,
            claims.sub
        )
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

        if !user.is_active {
            return Err(AppError::Unauthorized);
        }

        // ตรวจ token_version ให้ตรงกับ DB
        if claims.token_version != user.token_version {
            return Err(AppError::Unauthorized);
        }

        // ตรวจ iat กับ password_changed_at (ถ้ามี)
        if let Some(changed_at) = user.password_changed_at {
            if (claims.iat as i64) < changed_at.timestamp() {
                return Err(AppError::Unauthorized);
            }
        }

        Ok(AuthUser {
            id: claims.sub,
            username: user.username,
            role: user.role,
        })
    }
}

// sample handler
pub async fn me(user: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "id": user.id,
        "username": user.username,
        "role": user.role
    }))
}
