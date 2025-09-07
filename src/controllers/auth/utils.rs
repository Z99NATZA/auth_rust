use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::app::{error::AppError, result::AppResult};

// 32 ไบต์แบบสุ่ม + encode เป็น base64url (opaque token)
pub fn generate_refresh_token() -> AppResult<String> {
    let mut bytes = [0u8; 32];
    // ใน getrandom 0.3.x ใช้ fill()
    getrandom::fill(&mut bytes).map_err(|e| AppError::InternalError(format!("RNG failed: {:?}", e)))?;
    Ok(URL_SAFE_NO_PAD.encode(&bytes))
}

// แฮชฝั่งเซิร์ฟเวอร์ (HMAC-SHA256 ด้วย server secret) เก็บลง DB แทน token จริง
pub fn hash_refresh_token(token: &str, secret: &[u8]) -> AppResult<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret)?;
    mac.update(token.as_bytes());
    let result = mac.finalize().into_bytes();
    Ok(URL_SAFE_NO_PAD.encode(result))
}
