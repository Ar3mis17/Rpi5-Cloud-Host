-- Ar3mis Cloud initial schema

CREATE TABLE IF NOT EXISTS users (
    id BIGSERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role VARCHAR(20) NOT NULL DEFAULT 'user',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS sessions (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL,
    device_name VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMPTZ NOT NULL,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS files (
    id BIGSERIAL PRIMARY KEY,
    owner_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    parent_id BIGINT REFERENCES files(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    storage_key TEXT NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    checksum TEXT NOT NULL DEFAULT '',
    version INTEGER NOT NULL DEFAULT 1,
    is_folder BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS shares (
    id BIGSERIAL PRIMARY KEY,
    file_id BIGINT NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    owner_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    recipient_email VARCHAR(255),
    permissions VARCHAR(20) NOT NULL DEFAULT 'read',
    share_token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_storage (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    quota_bytes BIGINT NOT NULL DEFAULT 107374182400,
    used_bytes BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS activity_log (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id BIGINT,
    ip_address INET,
    user_agent TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_files_owner_parent ON files(owner_id, parent_id) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_sessions_user ON sessions(user_id) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_shares_token ON shares(share_token);
CREATE INDEX IF NOT EXISTS idx_activity_log_user ON activity_log(user_id, created_at DESC);
