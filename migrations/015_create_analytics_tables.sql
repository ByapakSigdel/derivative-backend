-- Migration: Create analytics and metrics tracking
-- Tracks compilation attempts, Arduino uploads, and system metrics

-- Create compilation logs table (for Arduino code generation tracking)
CREATE TABLE IF NOT EXISTS compilation_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL, -- 'success', 'error', 'warning'
    error_message TEXT,
    compilation_time_ms INTEGER, -- How long compilation took
    code_size_bytes INTEGER, -- Size of generated code
    node_count INTEGER, -- Number of nodes in project at time of compilation
    edge_count INTEGER, -- Number of edges
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create upload logs table (for Arduino upload tracking)
CREATE TABLE IF NOT EXISTS upload_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    project_id UUID NOT NULL REFERENCES user_projects(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,
    board_type VARCHAR(100), -- e.g., 'Arduino Uno', 'ESP32'
    port VARCHAR(100), -- Serial port used
    status VARCHAR(20) NOT NULL, -- 'success', 'error'
    error_message TEXT,
    upload_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create system metrics table (for overall platform statistics)
CREATE TABLE IF NOT EXISTS system_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_date DATE NOT NULL UNIQUE,
    total_users INTEGER NOT NULL DEFAULT 0,
    active_users INTEGER NOT NULL DEFAULT 0, -- Users who logged in that day
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
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_compilation_logs_project ON compilation_logs(project_id);
CREATE INDEX idx_compilation_logs_user ON compilation_logs(user_id);
CREATE INDEX idx_compilation_logs_status ON compilation_logs(status);
CREATE INDEX idx_compilation_logs_created ON compilation_logs(created_at DESC);

CREATE INDEX idx_upload_logs_project ON upload_logs(project_id);
CREATE INDEX idx_upload_logs_user ON upload_logs(user_id);
CREATE INDEX idx_upload_logs_status ON upload_logs(status);
CREATE INDEX idx_upload_logs_created ON upload_logs(created_at DESC);

CREATE INDEX idx_system_metrics_date ON system_metrics(metric_date DESC);

-- Function to log compilation attempt
CREATE OR REPLACE FUNCTION log_compilation(
    p_project_id UUID,
    p_user_id UUID,
    p_status VARCHAR,
    p_error_message TEXT DEFAULT NULL,
    p_compilation_time_ms INTEGER DEFAULT NULL,
    p_code_size_bytes INTEGER DEFAULT NULL,
    p_node_count INTEGER DEFAULT NULL,
    p_edge_count INTEGER DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_log_id UUID;
BEGIN
    INSERT INTO compilation_logs (
        project_id, user_id, status, error_message, 
        compilation_time_ms, code_size_bytes, node_count, edge_count
    ) VALUES (
        p_project_id, p_user_id, p_status, p_error_message,
        p_compilation_time_ms, p_code_size_bytes, p_node_count, p_edge_count
    ) RETURNING id INTO v_log_id;
    
    RETURN v_log_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log upload attempt
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

-- Function to update daily metrics (called by cron or manually)
CREATE OR REPLACE FUNCTION update_daily_metrics(p_date DATE DEFAULT CURRENT_DATE)
RETURNS VOID AS $$
BEGIN
    INSERT INTO system_metrics (
        metric_date,
        total_users,
        active_users,
        new_users,
        total_projects,
        new_projects,
        public_projects,
        total_compilations,
        successful_compilations,
        total_uploads,
        successful_uploads,
        total_views,
        total_likes,
        total_comments
    ) VALUES (
        p_date,
        (SELECT COUNT(*) FROM user_profiles),
        0, -- Will be updated by login tracking
        (SELECT COUNT(*) FROM user_profiles WHERE DATE(created_at) = p_date),
        (SELECT COUNT(*) FROM user_projects),
        (SELECT COUNT(*) FROM user_projects WHERE DATE(created_at) = p_date),
        (SELECT COUNT(*) FROM user_projects WHERE is_public = TRUE),
        (SELECT COUNT(*) FROM compilation_logs WHERE DATE(created_at) = p_date),
        (SELECT COUNT(*) FROM compilation_logs WHERE DATE(created_at) = p_date AND status = 'success'),
        (SELECT COUNT(*) FROM upload_logs WHERE DATE(created_at) = p_date),
        (SELECT COUNT(*) FROM upload_logs WHERE DATE(created_at) = p_date AND status = 'success'),
        (SELECT COALESCE(SUM(view_count), 0) FROM user_projects),
        (SELECT COALESCE(SUM(like_count), 0) FROM user_projects),
        (SELECT COALESCE(SUM(comment_count), 0) FROM user_projects)
    )
    ON CONFLICT (metric_date) DO UPDATE SET
        total_users = EXCLUDED.total_users,
        new_users = EXCLUDED.new_users,
        total_projects = EXCLUDED.total_projects,
        new_projects = EXCLUDED.new_projects,
        public_projects = EXCLUDED.public_projects,
        total_compilations = EXCLUDED.total_compilations,
        successful_compilations = EXCLUDED.successful_compilations,
        total_uploads = EXCLUDED.total_uploads,
        successful_uploads = EXCLUDED.successful_uploads,
        total_views = EXCLUDED.total_views,
        total_likes = EXCLUDED.total_likes,
        total_comments = EXCLUDED.total_comments,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;
