use axum::Json;
use axum::extract::FromRequestParts;
use axum::http::{request::Parts};
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
        let app = state.clone();

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized)?;

        let mut v = Validation::new(Algorithm::HS256);
        v.algorithms = vec![Algorithm::HS256];
        v.validate_exp = true;
        v.leeway = 30;

        let data = decode::<Claims>(
            bearer.token(),
            &DecodingKey::from_secret(app.jwt_secret.as_bytes()),
            &v,
        )?;

        Ok(AuthUser {
            id: data.claims.sub,
            username: data.claims.username,
            role: data.claims.role,
        })
    }
}

pub async fn me(user: AuthUser) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "id": user.id, "username": user.username, "role": user.role }))
}