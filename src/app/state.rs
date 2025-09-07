#![allow(dead_code)]

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::app::result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: Vec<u8>,
    pub jwt_issuer: String,
    pub jwt_audience: String,
    pub access_token_ttl: i64,
    pub refresh_secret: Vec<u8>,
}

impl AppState {
    pub async fn connect(database_url: &str) -> AppResult<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(pool)
    }
}