-- Migration: 003_create_user_projects
-- Description: Create user_projects table for visual programming projects

CREATE TYPE project_difficulty AS ENUM ('beginner', 'intermediate', 'advanced', 'expert');
CREATE TYPE project_category AS ENUM ('tutorial', 'game', 'simulation', 'art', 'music', 'utility', 'education', 'other');

CREATE TABLE user_projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    difficulty project_difficulty NOT NULL DEFAULT 'beginner',
    category project_category NOT NULL DEFAULT 'other',
    nodes JSONB NOT NULL DEFAULT '[]'::jsonb,
    edges JSONB NOT NULL DEFAULT '[]'::jsonb,
    materials TEXT[] DEFAULT ARRAY[]::TEXT[],
    learning_goals TEXT[] DEFAULT ARRAY[]::TEXT[],
    tags TEXT[] DEFAULT ARRAY[]::TEXT[],
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    featured BOOLEAN NOT NULL DEFAULT FALSE,
    view_count INTEGER NOT NULL DEFAULT 0,
    clone_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    search_vector TSVECTOR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

-- Indexes for project queries
CREATE INDEX idx_user_projects_user_id ON user_projects(user_id);
CREATE INDEX idx_user_projects_is_public ON user_projects(is_public);
CREATE INDEX idx_user_projects_featured ON user_projects(featured);
CREATE INDEX idx_user_projects_category ON user_projects(category);
CREATE INDEX idx_user_projects_difficulty ON user_projects(difficulty);
CREATE INDEX idx_user_projects_created_at ON user_projects(created_at DESC);
CREATE INDEX idx_user_projects_clone_count ON user_projects(clone_count DESC);
CREATE INDEX idx_user_projects_like_count ON user_projects(like_count DESC);
CREATE INDEX idx_user_projects_comment_count ON user_projects(comment_count DESC);
CREATE INDEX idx_user_projects_view_count ON user_projects(view_count DESC);
CREATE INDEX idx_user_projects_tags ON user_projects USING GIN(tags);

-- GIN index for full-text search
CREATE INDEX idx_user_projects_search_vector ON user_projects USING GIN(search_vector);

-- Composite indexes for common query patterns
CREATE INDEX idx_user_projects_public_featured ON user_projects(is_public, featured) WHERE is_public = TRUE;
CREATE INDEX idx_user_projects_user_public ON user_projects(user_id, is_public);

COMMENT ON TABLE user_projects IS 'Visual programming projects created by users';
COMMENT ON COLUMN user_projects.nodes IS 'JSON array of node definitions for the visual programming canvas';
COMMENT ON COLUMN user_projects.edges IS 'JSON array of edge connections between nodes';
COMMENT ON COLUMN user_projects.search_vector IS 'Pre-computed full-text search vector for title and description';
COMMENT ON COLUMN user_projects.published_at IS 'Timestamp when project was first made public';
