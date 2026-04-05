-- Migration: 004_create_project_likes
-- Description: Create project_likes table for user likes on projects

CREATE TABLE project_likes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Ensure each user can only like a project once
    CONSTRAINT unique_project_like UNIQUE(project_id, user_id)
);

-- Indexes for like queries
CREATE INDEX idx_project_likes_project_id ON project_likes(project_id);
CREATE INDEX idx_project_likes_user_id ON project_likes(user_id);
CREATE INDEX idx_project_likes_created_at ON project_likes(created_at DESC);

COMMENT ON TABLE project_likes IS 'Likes on projects by users';
COMMENT ON COLUMN project_likes.project_id IS 'The project being liked';
COMMENT ON COLUMN project_likes.user_id IS 'The user who liked the project';
