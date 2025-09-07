use std::sync::Arc;
use axum::{extract::State, http::{StatusCode}, response::Response};
use axum_extra::extract::CookieJar;
use cookie::Cookie;
use crate::{app::{result::AppResult, state::AppState}, controllers::auth::utils::hash_refresh_token};

pub async fn logout(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
) -> AppResult<Response> {
    // เพิกถอน refresh token ใน DB ถ้ามีคุกกี้
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

    // ให้ CookieJar สร้างคุกกี้ลบให้อัตโนมัติ (จะตั้ง Expires ในอดีต)
    let jar = jar.remove(
        Cookie::build("refresh_token")
            .path("/auth")
            .build()
    );

    // คืน NO_CONTENT + Set-Cookie (ลบ)
    Ok((jar, StatusCode::NO_CONTENT).into_response())

}