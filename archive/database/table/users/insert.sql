INSERT INTO users (username, email, password_hash, role)
VALUES (
    'demo_user',
    'demo@example.com',
    '$argon2id$v=19$m=19456,t=2,p=1$99/3ZKwFzBzaFWys61KS4A$YbvMfjtNqKc/d7uY6veri8izG04zidOO8VNZOJMdIeI', -- "password"
    'user'
);
