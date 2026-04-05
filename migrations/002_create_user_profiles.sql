-- Migration: 002_create_user_profiles
-- Description: Create user_profiles table for user authentication and profile data

CREATE TYPE user_type AS ENUM ('admin', 'user');

CREATE TABLE user_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL UNIQUE,
    full_name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    user_type user_type NOT NULL DEFAULT 'user',
    organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL,
    avatar_url VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    refresh_token VARCHAR(500),
    refresh_token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for user lookups
CREATE UNIQUE INDEX idx_user_profiles_email ON user_profiles(email);
CREATE INDEX idx_user_profiles_user_type ON user_profiles(user_type);
CREATE INDEX idx_user_profiles_organization_id ON user_profiles(organization_id);
CREATE INDEX idx_user_profiles_is_active ON user_profiles(is_active);

COMMENT ON TABLE user_profiles IS 'User accounts with authentication and profile information';
COMMENT ON COLUMN user_profiles.id IS 'Unique identifier for the user';
COMMENT ON COLUMN user_profiles.email IS 'User email address (unique, used for login)';
COMMENT ON COLUMN user_profiles.full_name IS 'User display name';
COMMENT ON COLUMN user_profiles.password_hash IS 'Argon2 hashed password';
COMMENT ON COLUMN user_profiles.user_type IS 'User role: admin or regular user';
COMMENT ON COLUMN user_profiles.organization_id IS 'Optional organization membership';
COMMENT ON COLUMN user_profiles.avatar_url IS 'URL to user avatar image';
COMMENT ON COLUMN user_profiles.is_active IS 'Whether the account is active';
COMMENT ON COLUMN user_profiles.refresh_token IS 'Current refresh token for JWT rotation';
