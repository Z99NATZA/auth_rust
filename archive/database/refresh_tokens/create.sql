CREATE TABLE refresh_tokens (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  token_hash TEXT NOT NULL,               -- เก็บ hash ของ refresh token (เช่น SHA-256)
  user_agent TEXT,                        -- อุปกรณ์/เบราว์เซอร์
  ip INET,                                -- ไอพีล่าสุด
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at TIMESTAMPTZ NOT NULL,        -- อายุของ refresh token
  revoked_at TIMESTAMPTZ                  -- ถ้าถูกเพิกถอน
);

CREATE INDEX IF NOT EXISTS idx_refresh_user ON refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_refresh_exp  ON refresh_tokens(expires_at);
