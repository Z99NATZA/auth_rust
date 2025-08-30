use sqlx::PgPool;

#[tokio::test]
#[ignore]
async fn test_db_connection() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPool::connect(&url).await?;

    sqlx::query("SELECT 1").execute(&pool).await?;
    Ok(())
}
