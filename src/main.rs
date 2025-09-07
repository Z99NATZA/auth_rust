mod app;
mod routers;
mod server;
mod controllers;
mod middleware;
mod utils;

use crate::app::result::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    server::run().await?;

    Ok(())
}