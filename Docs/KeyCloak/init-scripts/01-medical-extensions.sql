-- 医疗系统必需的PostgreSQL扩展
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- -- 启用行级安全
-- ALTER TABLE public.user_attribute ENABLE ROW LEVEL SECURITY;
-- ALTER TABLE public.user_session ENABLE ROW LEVEL SECURITY;

-- -- 医疗数据加密函数
-- CREATE OR REPLACE FUNCTION medical_encrypt_data(data TEXT) 
-- RETURNS BYTEA AS $$
-- BEGIN
--   RETURN pgp_sym_encrypt(data, current_setting('app.encryption_key'));
-- END;
-- $$ LANGUAGE plpgsql SECURITY DEFINER;

-- CREATE OR REPLACE FUNCTION medical_decrypt_data(data BYTEA) 
-- RETURNS TEXT AS $$
-- BEGIN
--   RETURN pgp_sym_decrypt(data, current_setting('app.encryption_key'));
-- END;
-- $$ LANGUAGE plpgsql SECURITY DEFINER;

-- -- 设置医疗合规参数
-- ALTER DATABASE keycloak SET app.encryption_key TO 'your_strong_encryption_key_here';
-- ALTER DATABASE keycloak SET timezone TO 'UTC';