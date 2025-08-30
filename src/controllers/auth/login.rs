use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::app::{result::AppResult, state::AppState};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<()> {
    
    println!("{payload:?}");
    Ok(())
}