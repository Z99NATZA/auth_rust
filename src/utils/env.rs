use crate::app::result::AppResult;

/*
| ----------------------------
| fn env_i64
| - let ttl_min = env_i64("ACCESS_TTL_MIN", 15)?;
| ----------------------------
*/
pub fn env_i64(name: &str, default: i64) -> AppResult<i64> {
    use std::env::{self, VarError};

    match env::var(name) {
        Ok(s) if !s.trim().is_empty() => {
            let v: i64 = s.trim().parse()?; // ✅ ตอนนี้ ? แปลงเป็น AppError ได้แล้ว
            Ok(v)
        }
        Err(VarError::NotPresent) => Ok(default), // ไม่มีตัวแปร -> ใช้ default
        Err(e) => Err(e.into()), // อื่น ๆ -> ส่งต่อเป็น AppError::EnvVarError
        _ => Ok(default), // พบแต่ค่าว่าง -> ใช้ default
    }
}
