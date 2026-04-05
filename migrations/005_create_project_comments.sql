-- Migration: 005_create_project_comments
-- Description: Create project_comments table for threaded comments on projects

CREATE TABLE project_comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    parent_id UUID REFERENCES project_comments(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    is_edited BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for comment queries
CREATE INDEX idx_project_comments_project_id ON project_comments(project_id);
CREATE INDEX idx_project_comments_user_id ON project_comments(user_id);
CREATE INDEX idx_project_comments_parent_id ON project_comments(parent_id);
CREATE INDEX idx_project_comments_created_at ON project_comments(created_at DESC);
CREATE INDEX idx_project_comments_not_deleted ON project_comments(project_id, created_at) WHERE is_deleted = FALSE;

COMMENT ON TABLE project_comments IS 'Comments on projects with threaded replies support';
COMMENT ON COLUMN project_comments.parent_id IS 'Parent comment ID for threaded replies (NULL for top-level comments)';
COMMENT ON COLUMN project_comments.is_deleted IS 'Soft delete flag - content hidden but structure preserved';
COMMENT ON COLUMN project_comments.is_edited IS 'Flag indicating if comment was edited after creation';
