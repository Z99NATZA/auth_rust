use std::{env, net::SocketAddr, sync::Arc};
use tokio::signal;
use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::app::state::AppState;
use crate::routers;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

pub async fn run() -> AppResult<()> {
    // -----------------------
    // โหลด .env ตอน dev เท่านั้น
    // -----------------------
    if cfg!(debug_assertions) {
        dotenv::dotenv()?;
    }

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::BadRequest("DATABASE_URL is not set".into()))?;

    // jwt_secret
    let jwt_secret_b64 = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::BadRequest("JWT_SECRET is not set".into()))?;
    let jwt_secret = STANDARD.decode(jwt_secret_b64)?;

    // refresh_secret
    let refresh_secret_b64 = std::env::var("REFRESH_SECRET")
        .map_err(|_| AppError::BadRequest("REFRESH_SECRET is not set".into()))?;
    let refresh_secret = STANDARD.decode(refresh_secret_b64)?;

    // issuer (ค่าเริ่มต้น "myapp")
    let jwt_issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "myapp".into());

    // audience (ค่าเริ่มต้น "myapp-client")
    let jwt_audience = std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "myapp-client".into());

    // access token TTL (วินาที) – ค่าเริ่มต้น 7200 = 2 ชั่วโมง
    let access_token_ttl: i64 = std::env::var("ACCESS_TOKEN_TTL")
        .unwrap_or_else(|_| "7200".into())
        .parse()
        .unwrap_or(7200);

    // -----------------------
    // เชื่อมต่อ Database
    // -----------------------
    let db = AppState::connect(&database_url).await?;

    // -----------------------
    // Shared AppState
    // -----------------------
    let state = Arc::new(AppState {
        db,
        jwt_secret,
        jwt_issuer,
        jwt_audience,
        access_token_ttl,
        refresh_secret
    });
  
    // -----------------------
    // Router + Server
    // -----------------------
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| AppError::BadRequest(format!("invalid address: {e}")))?;

    let app = routers::api(state);
    println!("App running on: {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = signal::ctrl_c().await;
            println!("App offline");
        })
        .await?;

    Ok(())
}
