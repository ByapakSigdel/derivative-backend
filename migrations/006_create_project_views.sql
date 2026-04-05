-- Migration: 006_create_project_views
-- Description: Create project_views table for tracking project views and analytics

CREATE TABLE project_views (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID REFERENCES user_profiles(id) ON DELETE SET NULL,
    view_duration INTEGER, -- Duration in seconds
    referrer VARCHAR(500),
    ip_address INET,
    viewed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for view analytics
CREATE INDEX idx_project_views_project_id ON project_views(project_id);
CREATE INDEX idx_project_views_user_id ON project_views(user_id);
CREATE INDEX idx_project_views_viewed_at ON project_views(viewed_at DESC);
CREATE INDEX idx_project_views_project_date ON project_views(project_id, viewed_at);

COMMENT ON TABLE project_views IS 'Analytics data for project views';
COMMENT ON COLUMN project_views.user_id IS 'The user who viewed (NULL for anonymous views)';
COMMENT ON COLUMN project_views.view_duration IS 'How long the user viewed the project in seconds';
COMMENT ON COLUMN project_views.referrer IS 'Where the user came from';
COMMENT ON COLUMN project_views.ip_address IS 'IP address for anonymous view tracking';
