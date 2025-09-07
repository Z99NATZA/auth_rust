use std::sync::Arc;
use axum::{
    extract::{State, FromRequestParts},
    body::Body,
    http::{Request},
    middleware::Next,
    response::Response,
};
use crate::app::{result::AppResult, state::AppState};
use crate::controllers::auth::me::AuthUser;

pub async fn auth_mw(
    State(app): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> AppResult<Response> {
    let (mut parts, body) = req.into_parts();

    let user = AuthUser::from_request_parts(&mut parts, &app).await?;

    req = Request::<Body>::from_parts(parts, body);
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}
