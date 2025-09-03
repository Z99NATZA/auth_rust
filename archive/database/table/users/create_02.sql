-- ----------------------
-- ตัวนี้ไม่ใช่ Create นะ แต่ตั้งชื่อให้เรี่ยงกันเฉย ๆ
-- ----------------------

-- เปิด extension ที่จำเป็น
CREATE EXTENSION IF NOT EXISTS pgcrypto;   -- สำหรับ gen_random_uuid()
CREATE EXTENSION IF NOT EXISTS citext;     -- สำหรับ case-insensitive

-- ปรับปรุงตาราง users
ALTER TABLE users
  ADD COLUMN token_version INTEGER NOT NULL DEFAULT 1,  -- ใช้สำหรับ JWT: เพิ่มค่าเมื่อ force logout ทั้งระบบ, token เดิมจะใช้ไม่ได้
  ADD COLUMN password_changed_at TIMESTAMPTZ,           -- เวลาที่ผู้ใช้เปลี่ยนรหัสผ่านครั้งล่าสุด (ใช้ตรวจ iat ของ JWT)
  ADD COLUMN email_verified_at TIMESTAMPTZ;             -- เวลาที่ผู้ใช้ยืนยันอีเมลแล้ว (NULL = ยังไม่ยืนยัน)

-- เปลี่ยนชนิด username/email เป็น CITEXT เพื่อ unique แบบไม่แคส
ALTER TABLE users
  ALTER COLUMN username TYPE CITEXT,
  ALTER COLUMN email TYPE CITEXT;