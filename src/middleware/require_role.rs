use std::collections::HashSet;
use axum::{body::Body, http::Request, middleware::Next, response::Response};
use futures::future::BoxFuture;
use crate::app::error::AppError;
use crate::controllers::auth::me::AuthUser;

/// สร้าง middleware checker สำหรับชุด role ที่อนุญาต
pub fn require_role(
    allowed: &'static [&'static str],
) -> impl Fn(Request<Body>, Next) -> BoxFuture<'static, Result<Response, AppError>> + Clone {
    let allowed_set: HashSet<&'static str> = allowed.iter().copied().collect();

    move |req: Request<Body>, next: Next| {
        let allowed_set = allowed_set.clone();

        Box::pin(async move {
            let user = req
                .extensions()
                .get::<AuthUser>()
                .cloned()
                .ok_or(AppError::Unauthorized)?;

            if !allowed_set.contains(user.role.as_str()) {
                return Err(AppError::Forbidden);
            }

            Ok(next.run(req).await)
        })
    }
}
