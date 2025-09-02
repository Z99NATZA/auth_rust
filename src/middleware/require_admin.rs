use axum::{body::Body, http::Request, middleware::Next, response::Response};
use crate::app::error::AppError;
use crate::controllers::auth::me::AuthUser;

pub async fn require_admin(
    req: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let user = req.extensions().get::<AuthUser>()
        .cloned()
        .ok_or(AppError::Unauthorized)?;

    if user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(next.run(req).await)
}
