use std::env;

fn load_env() {
    dotenv::dotenv().ok();
}

#[test]
#[ignore]
fn test_env_host() {
    load_env();
    let ok = env::var("HOST").map(|h| !h.is_empty()).unwrap_or(false);
    println!("HOST ok? {}", ok);
    assert!(ok, "HOST must be set and not empty");
}

#[test]
#[ignore]
fn test_env_port() {
    load_env();
    let ok = env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .map(|p| p > 0)
        .unwrap_or(false);
    println!("PORT ok? {}", ok);
    assert!(ok, "PORT must be a valid number > 0");
}

#[test]
#[ignore]
fn test_env_database_url() {
    load_env();
    let ok = env::var("DATABASE_URL")
        .map(|url| url.starts_with("postgres://"))
        .unwrap_or(false);
    println!("DATABASE_URL ok? {}", ok);
    assert!(ok, "DATABASE_URL must start with postgres://");
}

#[test]
#[ignore]
fn test_env_jwt_secret() {
    load_env();
    let ok = env::var("JWT_SECRET")
        .map(|s| s.len() >= 32)
        .unwrap_or(false);
    println!("JWT_SECRET ok? {}", ok);
    assert!(ok, "JWT_SECRET must be set and >= 32 chars");
}
