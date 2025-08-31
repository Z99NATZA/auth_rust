#![allow(dead_code)]

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::app::result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
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