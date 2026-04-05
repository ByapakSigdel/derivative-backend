-- Migration: 001_create_organizations
-- Description: Create organizations table for grouping users

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for organization name lookups
CREATE INDEX idx_organizations_name ON organizations(name);

COMMENT ON TABLE organizations IS 'Organizations that users can belong to';
COMMENT ON COLUMN organizations.id IS 'Unique identifier for the organization';
COMMENT ON COLUMN organizations.name IS 'Display name of the organization';
COMMENT ON COLUMN organizations.description IS 'Optional description of the organization';
