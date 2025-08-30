use std::{env, net::SocketAddr, sync::Arc};
use sqlx::postgres::PgPoolOptions;
use tokio::signal;

use crate::app::error::AppError;
use crate::app::result::AppResult;
use crate::app::state::AppState;
use crate::routers::api;

pub async fn run() -> AppResult<()> {
    // -----------------------
    // โหลด .env ตอน dev เท่านั้น
    // -----------------------
    if cfg!(debug_assertions) {
        dotenv::dotenv()?;
    }

    // -----------------------
    // เตรียม Database
    // -----------------------
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| AppError::InternalError("DATABASE_URL not set".into()))?;

    // -----------------------
    // เชื่อมต่อ Database
    // -----------------------
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // -----------------------
    // Shared AppState
    // -----------------------
    let state = Arc::new(AppState {
        db: pool,
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
        .map_err(|e| AppError::BadRequest(format!("Invalid HOST/PORT: {e}")))?;

    let app = api(state);
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
