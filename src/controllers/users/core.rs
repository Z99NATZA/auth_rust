use std::sync::Arc;

use axum::{Json, extract::State};
use serde::Serialize;
use sqlx::prelude::FromRow;
use uuid::Uuid;

use crate::app::{result::AppResult, state::AppState};

#[derive(Debug, Serialize, FromRow)]
pub struct UsersResponse {
    id: Uuid,
    username: String,
    role: Option<String>,
}

pub async fn list_users(
    State(state): State<Arc<AppState>>
) -> AppResult<Json<Vec<UsersResponse>>> {
    let rows = sqlx::query_as::<_, UsersResponse>(
        r#"SELECT id, username, COALESCE(role, 'user') AS role FROM users"#
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(rows))
}