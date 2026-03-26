-- 微信小程序后端数据库初始化脚本
-- 创建时间: 2026-03-26

-- 启用 UUID 扩展
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 用户表
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    openid VARCHAR(64) NOT NULL UNIQUE,
    unionid VARCHAR(64),
    phone VARCHAR(20),
    nickname VARCHAR(64),
    avatar_url VARCHAR(512),
    gender SMALLINT DEFAULT 0 CHECK (gender IN (0, 1, 2)), -- 0:未知 1:男 2:女
    status SMALLINT DEFAULT 1 CHECK (status IN (0, 1)),    -- 0:禁用 1:正常
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 会话表 (存储微信session_key)
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_key VARCHAR(64) NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 登录日志表
CREATE TABLE IF NOT EXISTS login_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    login_type VARCHAR(20) NOT NULL,                        -- login/phone_refresh
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 用户信息变更日志表 (用于审计)
CREATE TABLE IF NOT EXISTS user_profile_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    field_name VARCHAR(32) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_users_openid ON users(openid);
CREATE INDEX IF NOT EXISTS idx_users_phone ON users(phone);
CREATE INDEX IF NOT EXISTS idx_users_status ON users(status);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);
CREATE INDEX IF NOT EXISTS idx_login_logs_user_id ON login_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_login_logs_created_at ON login_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_user_profile_logs_user_id ON user_profile_logs(user_id);

-- 更新时间触发器函数
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 用户表更新时间触发器
DROP TRIGGER IF EXISTS update_users_updated_at ON users;
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- 清理过期会话的函数
CREATE OR REPLACE FUNCTION cleanup_expired_sessions()
RETURNS void AS $$
BEGIN
    DELETE FROM sessions WHERE expires_at < CURRENT_TIMESTAMP;
END;
$$ LANGUAGE plpgsql;

-- 注释
COMMENT ON TABLE users IS '用户表';
COMMENT ON COLUMN users.openid IS '微信用户唯一标识';
COMMENT ON COLUMN users.unionid IS '微信开放平台唯一标识';
COMMENT ON COLUMN users.gender IS '性别: 0-未知, 1-男, 2-女';
COMMENT ON COLUMN users.status IS '状态: 0-禁用, 1-正常';

COMMENT ON TABLE sessions IS '微信会话表，存储session_key';
COMMENT ON COLUMN sessions.session_key IS '微信会话密钥，用于解密敏感数据';

COMMENT ON TABLE login_logs IS '登录日志表';
COMMENT ON COLUMN login_logs.login_type IS '登录类型: login-登录, phone_refresh-刷新手机号';
