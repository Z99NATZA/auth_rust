use std::sync::Arc;
use axum::{extract::State, http::{StatusCode}};
use cookie::Cookie;
use crate::{app::{result::AppResult, state::AppState}, controllers::auth::utils::hash_refresh_token};
use axum_extra::extract::cookie::CookieJar;

pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> AppResult<impl axum::response::IntoResponse> {
    // revoke ใน DB ถ้ามี cookie
    if let Some(c) = jar.get("refresh_token") {
        let hash = hash_refresh_token(c.value(), &state.refresh_secret)?;
        
        sqlx::query!(
            "UPDATE refresh_tokens SET revoked_at = now()
             WHERE token_hash = $1 AND revoked_at IS NULL",
            hash
        )
        .execute(&state.db)
        .await?;
    }

    // ลบคุกกี้ด้วย CookieJar (ต้องตั้ง path ให้ตรงกับตอน set)
    let jar = jar.remove(
        Cookie::build("refresh_token")
            .path("/auth")
            .build(),
    );

    // คืน NO_CONTENT + Set-Cookie (ลบทิ้ง) โดยไม่ต้อง .into_response()
    Ok((jar, StatusCode::NO_CONTENT))
}