-- Update test user password hash with correct Argon2id parameters
-- Password: TestPass123!
-- Generated with: m=65536,t=3,p=4

UPDATE users 
SET password_hash = '$argon2id$v=19$m=65536,t=3,p=4$c236Pt+gD8usnvIe3ZJqqw$r9Q/yNFUsqR8BK7UdRYIdHtAcy6iJkd4qkkZKI/47hY',
    updated_at = NOW()
WHERE username = 'testuser';
