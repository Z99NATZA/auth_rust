CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL,                -- ค่าที่ hash แล้ว (ไม่เก็บ plain)
    expires_at TIMESTAMPTZ NOT NULL,    -- วันหมดอายุ
    created_at TIMESTAMPTZ DEFAULT now()
);
