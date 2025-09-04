-- เปิด extension ที่จำเป็น
CREATE EXTENSION IF NOT EXISTS pgcrypto;   -- สำหรับ gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS citext;     -- สำหรับ case-insensitive

-- ตาราง users
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),    -- ใช้ UUID เป็น PK
    username CITEXT UNIQUE NOT NULL,                  -- username ห้ามซ้ำ (ไม่แคส)
    email CITEXT UNIQUE NOT NULL,                     -- email ห้ามซ้ำ (ไม่แคส)
    password_hash TEXT NOT NULL,                      -- เก็บ password hash (ไม่เก็บ plain)
    role VARCHAR(20) NOT NULL DEFAULT 'user',         -- สิทธิ์ เช่น user/admin (ควร constrain ให้เป็นชุดที่กำหนด)
    is_active BOOLEAN NOT NULL DEFAULT TRUE,          -- ใช้ปิดบัญชีได้
    token_version INTEGER NOT NULL DEFAULT 1,         -- ใช้สำหรับ JWT: เพิ่มค่าเมื่อ force logout ทั้งระบบ
    password_changed_at TIMESTAMPTZ,                  -- เวลาที่ผู้ใช้เปลี่ยนรหัสผ่านล่าสุด (ตรวจ iat ของ JWT)
    email_verified_at TIMESTAMPTZ,                    -- เวลาที่ผู้ใช้ยืนยันอีเมลแล้ว (NULL = ยังไม่ยืนยัน)
    failed_login_attempts INTEGER NOT NULL DEFAULT 0, -- จำนวนครั้งที่ login ล้มเหลว
    locked_until TIMESTAMPTZ,                         -- ถ้ามีค่า = บัญชีถูกล็อกชั่วคราว
    last_login_at TIMESTAMPTZ,                        -- เวลาที่ login สำเร็จครั้งล่าสุด
    mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,       -- เปิดใช้ MFA หรือไม่
    mfa_totp_secret BYTEA,                            -- เก็บ TOTP secret (ควรเข้ารหัสด้วย pgcrypto)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),    -- วันที่สมัคร
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()     -- อัปเดตล่าสุด
);
