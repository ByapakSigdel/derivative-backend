-- Migration: Create all collaboration and analytics tables
-- This is a comprehensive migration that creates all necessary tables and functions
-- Version: 016 (combined)

-- ============================================
-- PART 1: COLLABORATION TABLES
-- ============================================

-- Create project_collaborators table
CREATE TABLE IF NOT EXISTS project_collaborators (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    role VARCHAR(20) NOT NULL DEFAULT 'editor',
    invited_by UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    invited_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    accepted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(project_id, user_id)
);

-- Create invite tokens table
CREATE TABLE IF NOT EXISTS project_invite_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    token VARCHAR(64) NOT NULL UNIQUE,
    role VARCHAR(20) NOT NULL DEFAULT 'editor',
    created_by UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    max_uses INTEGER,
    uses_count INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- ============================================
-- PART 2: ANALYTICS TABLES
-- ============================================

-- Create compilation logs table
CREATE TABLE IF NOT EXISTS compilation_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES user_projects(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    compilation_time_ms INTEGER,
    code_size_bytes INTEGER,
    node_count INTEGER,
    edge_count INTEGER,
    board_type VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create upload logs table
CREATE TABLE IF NOT EXISTS upload_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID REFERENCES user_projects(id) ON DELETE SET NULL,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    board_type VARCHAR(100),
    port VARCHAR(100),
    status VARCHAR(20) NOT NULL,
    error_message TEXT,
    upload_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create daily system metrics table
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_date DATE NOT NULL UNIQUE,
    total_users INTEGER NOT NULL DEFAULT 0,
    active_users INTEGER NOT NULL DEFAULT 0,
    new_users INTEGER NOT NULL DEFAULT 0,
    total_projects INTEGER NOT NULL DEFAULT 0,
    new_projects INTEGER NOT NULL DEFAULT 0,
    public_projects INTEGER NOT NULL DEFAULT 0,
    total_compilations INTEGER NOT NULL DEFAULT 0,
    successful_compilations INTEGER NOT NULL DEFAULT 0,
    total_uploads INTEGER NOT NULL DEFAULT 0,
    successful_uploads INTEGER NOT NULL DEFAULT 0,
    total_views INTEGER NOT NULL DEFAULT 0,
    total_likes INTEGER NOT NULL DEFAULT 0,
    total_comments INTEGER NOT NULL DEFAULT 0,
    total_collaborators INTEGER NOT NULL DEFAULT 0,
    active_collaboration_sessions INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create user activity log for tracking logins and actions
CREATE TABLE IF NOT EXISTS user_activity_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    action_type VARCHAR(50) NOT NULL, -- 'login', 'logout', 'project_create', 'project_edit', etc.
    metadata JSONB,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create collaboration sessions table for real-time tracking
CREATE TABLE IF NOT EXISTS collaboration_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    session_start TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    session_end TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- ============================================
-- PART 3: INDEXES
-- ============================================

CREATE INDEX IF NOT EXISTS idx_project_collaborators_project ON project_collaborators(project_id);
CREATE INDEX IF NOT EXISTS idx_project_collaborators_user ON project_collaborators(user_id);
CREATE INDEX IF NOT EXISTS idx_project_invite_tokens_project ON project_invite_tokens(project_id);
CREATE INDEX IF NOT EXISTS idx_project_invite_tokens_token ON project_invite_tokens(token);
CREATE INDEX IF NOT EXISTS idx_project_invite_tokens_active ON project_invite_tokens(is_active) WHERE is_active = TRUE;

CREATE INDEX IF NOT EXISTS idx_compilation_logs_project ON compilation_logs(project_id);
CREATE INDEX IF NOT EXISTS idx_compilation_logs_user ON compilation_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_compilation_logs_status ON compilation_logs(status);
CREATE INDEX IF NOT EXISTS idx_compilation_logs_created ON compilation_logs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_upload_logs_project ON upload_logs(project_id);
CREATE INDEX IF NOT EXISTS idx_upload_logs_user ON upload_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_upload_logs_created ON upload_logs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_system_metrics_date ON system_metrics(metric_date DESC);

CREATE INDEX IF NOT EXISTS idx_user_activity_logs_user ON user_activity_logs(user_id);
CREATE INDEX IF NOT EXISTS idx_user_activity_logs_action ON user_activity_logs(action_type);
CREATE INDEX IF NOT EXISTS idx_user_activity_logs_created ON user_activity_logs(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_collaboration_sessions_project ON collaboration_sessions(project_id);
CREATE INDEX IF NOT EXISTS idx_collaboration_sessions_user ON collaboration_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_collaboration_sessions_active ON collaboration_sessions(is_active) WHERE is_active = TRUE;

-- ============================================
-- PART 4: FUNCTIONS
-- ============================================

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
    
    -- Check if project is public
    IF EXISTS (
        SELECT 1 FROM user_projects 
        WHERE id = p_project_id AND is_public = TRUE
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

-- Trigger to update timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply trigger to tables with updated_at
DROP TRIGGER IF EXISTS trigger_update_collaborators_timestamp ON project_collaborators;
CREATE TRIGGER trigger_update_collaborators_timestamp
    BEFORE UPDATE ON project_collaborators
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS trigger_update_system_metrics_timestamp ON system_metrics;
CREATE TRIGGER trigger_update_system_metrics_timestamp
    BEFORE UPDATE ON system_metrics
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================
-- PART 5: ANALYTICS FUNCTIONS
-- ============================================

-- Function to get dashboard metrics
CREATE OR REPLACE FUNCTION get_dashboard_metrics()
RETURNS TABLE (
    total_users BIGINT,
    total_projects BIGINT,
    public_projects BIGINT,
    private_projects BIGINT,
    total_organizations BIGINT,
    total_views BIGINT,
    total_likes BIGINT,
    total_comments BIGINT,
    total_compilations BIGINT,
    successful_compilations BIGINT,
    total_uploads BIGINT,
    successful_uploads BIGINT,
    featured_projects BIGINT,
    total_collaborators BIGINT,
    active_sessions BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM user_profiles)::BIGINT as total_users,
        (SELECT COUNT(*) FROM user_projects)::BIGINT as total_projects,
        (SELECT COUNT(*) FROM user_projects WHERE is_public = TRUE)::BIGINT as public_projects,
        (SELECT COUNT(*) FROM user_projects WHERE is_public = FALSE)::BIGINT as private_projects,
        (SELECT COUNT(*) FROM organizations)::BIGINT as total_organizations,
        (SELECT COALESCE(SUM(view_count), 0) FROM user_projects)::BIGINT as total_views,
        (SELECT COALESCE(SUM(like_count), 0) FROM user_projects)::BIGINT as total_likes,
        (SELECT COALESCE(SUM(comment_count), 0) FROM user_projects)::BIGINT as total_comments,
        (SELECT COUNT(*) FROM compilation_logs)::BIGINT as total_compilations,
        (SELECT COUNT(*) FROM compilation_logs WHERE status = 'success')::BIGINT as successful_compilations,
        (SELECT COUNT(*) FROM upload_logs)::BIGINT as total_uploads,
        (SELECT COUNT(*) FROM upload_logs WHERE status = 'success')::BIGINT as successful_uploads,
        (SELECT COUNT(*) FROM user_projects WHERE is_featured = TRUE)::BIGINT as featured_projects,
        (SELECT COUNT(*) FROM project_collaborators WHERE accepted_at IS NOT NULL)::BIGINT as total_collaborators,
        (SELECT COUNT(*) FROM collaboration_sessions WHERE is_active = TRUE)::BIGINT as active_sessions;
END;
$$ LANGUAGE plpgsql;

-- Function to get time series metrics
CREATE OR REPLACE FUNCTION get_time_series_metrics(
    p_days INTEGER DEFAULT 30,
    p_metric VARCHAR DEFAULT 'users'
)
RETURNS TABLE (
    date DATE,
    value BIGINT
) AS $$
BEGIN
    IF p_metric = 'users' THEN
        RETURN QUERY
        SELECT DATE(created_at), COUNT(*)::BIGINT
        FROM user_profiles
        WHERE created_at >= CURRENT_DATE - p_days
        GROUP BY DATE(created_at)
        ORDER BY DATE(created_at);
    ELSIF p_metric = 'projects' THEN
        RETURN QUERY
        SELECT DATE(created_at), COUNT(*)::BIGINT
        FROM user_projects
        WHERE created_at >= CURRENT_DATE - p_days
        GROUP BY DATE(created_at)
        ORDER BY DATE(created_at);
    ELSIF p_metric = 'compilations' THEN
        RETURN QUERY
        SELECT DATE(created_at), COUNT(*)::BIGINT
        FROM compilation_logs
        WHERE created_at >= CURRENT_DATE - p_days
        GROUP BY DATE(created_at)
        ORDER BY DATE(created_at);
    ELSIF p_metric = 'views' THEN
        RETURN QUERY
        SELECT DATE(created_at), COUNT(*)::BIGINT
        FROM project_views
        WHERE created_at >= CURRENT_DATE - p_days
        GROUP BY DATE(created_at)
        ORDER BY DATE(created_at);
    ELSE
        RETURN QUERY SELECT NULL::DATE, 0::BIGINT WHERE FALSE;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to log compilation
CREATE OR REPLACE FUNCTION log_compilation(
    p_project_id UUID,
    p_user_id UUID,
    p_status VARCHAR,
    p_error_message TEXT DEFAULT NULL,
    p_compilation_time_ms INTEGER DEFAULT NULL,
    p_code_size_bytes INTEGER DEFAULT NULL,
    p_node_count INTEGER DEFAULT NULL,
    p_edge_count INTEGER DEFAULT NULL,
    p_board_type VARCHAR DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
BEGIN
    INSERT INTO compilation_logs (
        project_id, user_id, status, error_message, 
        compilation_time_ms, code_size_bytes, node_count, edge_count, board_type
    ) VALUES (
        p_project_id, p_user_id, p_status, p_error_message,
        p_compilation_time_ms, p_code_size_bytes, p_node_count, p_edge_count, p_board_type
    ) RETURNING id INTO v_log_id;
    
    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log upload
CREATE OR REPLACE FUNCTION log_upload(
    p_project_id UUID,
    p_user_id UUID,
    p_board_type VARCHAR DEFAULT NULL,
    p_port VARCHAR DEFAULT NULL,
    p_status VARCHAR DEFAULT 'success',
    p_error_message TEXT DEFAULT NULL,
    p_upload_time_ms INTEGER DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
BEGIN
    INSERT INTO upload_logs (
        project_id, user_id, board_type, port, status, error_message, upload_time_ms
    ) VALUES (
        p_project_id, p_user_id, p_board_type, p_port, p_status, p_error_message, p_upload_time_ms
    ) RETURNING id INTO v_log_id;
    
    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log user activity
CREATE OR REPLACE FUNCTION log_user_activity(
    p_user_id UUID,
    p_action_type VARCHAR,
    p_metadata JSONB DEFAULT NULL,
    p_ip_address VARCHAR DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
BEGIN
    INSERT INTO user_activity_logs (
        user_id, action_type, metadata, ip_address, user_agent
    ) VALUES (
        p_user_id, p_action_type, p_metadata, p_ip_address, p_user_agent
    ) RETURNING id INTO v_log_id;
    
    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to start collaboration session
CREATE OR REPLACE FUNCTION start_collaboration_session(
    p_project_id UUID,
    p_user_id UUID
)
RETURNS UUID AS $$
DECLARE
    v_session_id UUID;
BEGIN
    -- End any existing active sessions for this user on this project
    UPDATE collaboration_sessions 
    SET is_active = FALSE, 
        session_end = NOW(),
        duration_seconds = EXTRACT(EPOCH FROM (NOW() - session_start))::INTEGER
    WHERE project_id = p_project_id 
    AND user_id = p_user_id 
    AND is_active = TRUE;
    
    -- Start new session
    INSERT INTO collaboration_sessions (project_id, user_id)
    VALUES (p_project_id, p_user_id)
    RETURNING id INTO v_session_id;
    
    RETURN v_session_id;
END;
$$ LANGUAGE plpgsql;

-- Function to end collaboration session
CREATE OR REPLACE FUNCTION end_collaboration_session(
    p_project_id UUID,
    p_user_id UUID
)
RETURNS VOID AS $$
BEGIN
    UPDATE collaboration_sessions 
    SET is_active = FALSE, 
        session_end = NOW(),
        duration_seconds = EXTRACT(EPOCH FROM (NOW() - session_start))::INTEGER
    WHERE project_id = p_project_id 
    AND user_id = p_user_id 
    AND is_active = TRUE;
END;
$$ LANGUAGE plpgsql;