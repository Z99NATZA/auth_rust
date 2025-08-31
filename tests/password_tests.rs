use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, PasswordHash, rand_core::OsRng};

#[tokio::test]
#[ignore]
async fn test_verify_password() {
    let password = b"password";
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    // hash
    let password_hash = argon2.hash_password(password, &salt).unwrap().to_string();

    // parse กลับมาเป็น PasswordHash
    let parsed_hash = PasswordHash::new(&password_hash).unwrap();

    // verify
    match argon2.verify_password(password, &parsed_hash) {
        Ok(_) => println!("Ok Password ตรงกัน"),
        Err(_) => println!("Failed Password ไม่ตรง"),
    }

    // เพิ่ม assert เพื่อให้ test fail ถ้า verify ไม่ผ่าน
    assert!(argon2.verify_password(password, &parsed_hash).is_ok());
}
