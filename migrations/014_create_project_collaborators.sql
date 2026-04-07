-- Migration: Create project collaborators table for collaboration invites
-- This enables multiple users to edit the same project in real-time

-- Create project_collaborators table
CREATE TABLE IF NOT EXISTS project_collaborators (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'editor', -- 'owner', 'editor', 'viewer'
    invited_by UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    invited_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    accepted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Prevent duplicate collaborators
    UNIQUE(project_id, user_id)
);

-- Create invite tokens table for shareable links
CREATE TABLE IF NOT EXISTS project_invite_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE, -- Random token for URL
    role VARCHAR(20) NOT NULL DEFAULT 'editor',
    created_by UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    max_uses INTEGER, -- NULL means unlimited
    uses_count INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMP WITH TIME ZONE, -- NULL means never expires
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Indexes for performance
CREATE INDEX idx_project_collaborators_project ON project_collaborators(project_id);
CREATE INDEX idx_project_collaborators_user ON project_collaborators(user_id);
CREATE INDEX idx_project_invite_tokens_project ON project_invite_tokens(project_id);
CREATE INDEX idx_project_invite_tokens_token ON project_invite_tokens(token);
CREATE INDEX idx_project_invite_tokens_active ON project_invite_tokens(is_active) WHERE is_active = TRUE;

-- Function to check if user can access project
CREATE OR REPLACE FUNCTION can_user_access_project(p_project_id UUID, p_user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    -- Check if user is owner
    IF EXISTS (
        SELECT 1 FROM user_projects 
        WHERE id = p_project_id AND user_id = p_user_id
    ) THEN
        RETURN TRUE;
    END IF;
    
    -- Check if user is a collaborator
    IF EXISTS (
        SELECT 1 FROM project_collaborators 
        WHERE project_id = p_project_id 
        AND user_id = p_user_id 
        AND accepted_at IS NOT NULL
    ) THEN
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to check if user can edit project
CREATE OR REPLACE FUNCTION can_user_edit_project(p_project_id UUID, p_user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    -- Check if user is owner
    IF EXISTS (
        SELECT 1 FROM user_projects 
        WHERE id = p_project_id AND user_id = p_user_id
    ) THEN
        RETURN TRUE;
    END IF;
    
    -- Check if user is an editor collaborator
    IF EXISTS (
        SELECT 1 FROM project_collaborators 
        WHERE project_id = p_project_id 
        AND user_id = p_user_id 
        AND role IN ('owner', 'editor')
        AND accepted_at IS NOT NULL
    ) THEN
        RETURN TRUE;
    END IF;
    
    RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_collaborators_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_collaborators_timestamp
    BEFORE UPDATE ON project_collaborators
    FOR EACH ROW
    EXECUTE FUNCTION update_collaborators_timestamp();
