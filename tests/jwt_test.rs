use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Claims {
    sub: String,
    exp: usize,
}

#[test]
#[ignore]
fn test_jwt_secret_encode_decode() {
    // โหลด JWT_SECRET จาก env (.env ต้องถูกโหลดด้วย dotenv ใน dev test)
    dotenv::dotenv().ok();
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let claims = Claims {
        sub: "test-user".to_string(),
        exp: 2000000000, // timestamp อนาคต
    };

    // สร้าง token
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    ).expect("failed to encode");

    eprintln!("Generated token: {}", token);

    // decode token กลับมา
    let data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    ).expect("failed to decode");

    assert_eq!(data.claims, claims);
}
