CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),   -- ใช้ UUID เป็น PK
    username VARCHAR(50) UNIQUE NOT NULL,            -- username ห้ามซ้ำ
    email VARCHAR(100) UNIQUE NOT NULL,              -- email ไว้ยืนยัน/ติดต่อ
    password_hash TEXT NOT NULL,                     -- เก็บ password hash (ไม่เก็บ plain)
    role VARCHAR(20) DEFAULT 'user',                 -- สิทธิ์ เช่น user/admin
    is_active BOOLEAN DEFAULT TRUE,                  -- ใช้ปิดบัญชีได้
    created_at TIMESTAMPTZ DEFAULT now(),            -- วันที่สมัคร
    updated_at TIMESTAMPTZ DEFAULT now()             -- อัปเดตล่าสุด
);
