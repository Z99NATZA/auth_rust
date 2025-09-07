use axum::{Router, http::{HeaderValue, Method, header}, middleware::{from_fn_with_state, from_fn}};
use tower_http::cors::CorsLayer;
use std::sync::Arc;
use crate::{app::state::AppState, controllers::auth::refresh_token::refresh, middleware::{auth::auth_mw, require_role::require_role}};
use axum::routing::{post, get};
use crate::controllers::auth::login;
use crate::controllers::auth::me;
use crate::controllers::users::core::list_users;

pub fn api(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(HeaderValue::from_static("http://localhost:3000"))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT])
        .allow_credentials(true);

    let public = Router::new()
        .route("/auth/login", post(login::login))
        .route("/auth/refresh", post(refresh))
        ;

    let admin = Router::new()
        .route("/users", get(list_users))
        .route_layer(from_fn(require_role(&["admin"])));

    let authed = Router::new()
        .route("/auth/me", get(me::me))
        .nest("/api", admin)
        .route_layer(from_fn_with_state(state.clone(), auth_mw));

    public.merge(authed)
        .layer(cors)
        .with_state(state)
} 
