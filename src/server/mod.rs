use std::{env, net::SocketAddr, sync::Arc};
use tokio::signal;
use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::app::state::AppState;
use crate::routers;

pub async fn run() -> AppResult<()> {
    // -----------------------
    // โหลด .env ตอน dev เท่านั้น
    // -----------------------
    if cfg!(debug_assertions) {
        dotenv::dotenv()?;
    }

    let database_url = std::env::var("DATABASE_URL")?;
    // let jwt_secret = std::env::var("JWT_SECRET")?;
    
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| AppError::BadRequest("JWT_SECRET is not set".into()))?;

    // แนะนำอย่างน้อย 32 ไบต์ขึ้นไป
    if jwt_secret.len() < 32 {
        eprintln!("WARNING: JWT_SECRET is too short (<32 bytes). Use a longer, random secret.");
    }

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
            eprintln!("App offline");
        })
        .await?;

    Ok(())
}
