mod app;
mod routers;
mod server;
mod controllers;

use argon2::{
    Argon2, PasswordHash, password_hash::{
        PasswordHasher, SaltString, rand_core::OsRng,
        PasswordVerifier
    }
};


use crate::app::result::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    let password = b"password";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password, &salt)?.to_string();

    let parsed_hash = PasswordHash::new(&password_hash)?;
    
    match Argon2::default().verify_password(password, &parsed_hash) {
        Ok(_) => println!("Ok Password ตรงกัน"),
        Err(_) => println!("Failed Password ไม่ตรง"),
    }
        
    server::run().await?;

    Ok(())
}