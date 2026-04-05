-- Migration: 009_create_indexes
-- Description: Create additional indexes and views for performance

-- View: Public projects with user and organization data pre-joined
CREATE OR REPLACE VIEW public_projects_with_users AS
SELECT 
    p.id,
    p.user_id,
    p.title,
    p.description,
    p.difficulty,
    p.category,
    p.nodes,
    p.edges,
    p.materials,
    p.learning_goals,
    p.tags,
    p.is_public,
    p.featured,
    p.view_count,
    p.clone_count,
    p.like_count,
    p.comment_count,
    p.created_at,
    p.updated_at,
    p.published_at,
    u.id AS author_id,
    u.email AS author_email,
    u.full_name AS author_name,
    u.avatar_url AS author_avatar,
    o.id AS organization_id,
    o.name AS organization_name
FROM user_projects p
INNER JOIN user_profiles u ON p.user_id = u.id
LEFT JOIN organizations o ON u.organization_id = o.id
WHERE p.is_public = TRUE AND u.is_active = TRUE;

COMMENT ON VIEW public_projects_with_users IS 'Pre-joined view of public projects with author and organization information';

-- Additional composite indexes for common query patterns

-- Index for featured public projects ordered by popularity
CREATE INDEX IF NOT EXISTS idx_user_projects_featured_popular 
ON user_projects(featured DESC, like_count DESC, view_count DESC) 
WHERE is_public = TRUE;

-- Index for recent public projects
CREATE INDEX IF NOT EXISTS idx_user_projects_public_recent 
ON user_projects(created_at DESC) 
WHERE is_public = TRUE;

-- Index for user's draft projects
CREATE INDEX IF NOT EXISTS idx_user_projects_user_drafts 
ON user_projects(user_id, created_at DESC) 
WHERE is_public = FALSE;

-- Index for category browsing
CREATE INDEX IF NOT EXISTS idx_user_projects_category_popular 
ON user_projects(category, like_count DESC) 
WHERE is_public = TRUE;

-- Index for difficulty filtering
CREATE INDEX IF NOT EXISTS idx_user_projects_difficulty_recent 
ON user_projects(difficulty, created_at DESC) 
WHERE is_public = TRUE;

-- Partial index for active users
CREATE INDEX IF NOT EXISTS idx_user_profiles_active 
ON user_profiles(email, full_name) 
WHERE is_active = TRUE;

-- Index for comment threads
CREATE INDEX IF NOT EXISTS idx_project_comments_thread 
ON project_comments(project_id, parent_id NULLS FIRST, created_at ASC) 
WHERE is_deleted = FALSE;

-- Index for user's recent likes
CREATE INDEX IF NOT EXISTS idx_project_likes_user_recent 
ON project_likes(user_id, created_at DESC);

-- Index for recent project views
CREATE INDEX IF NOT EXISTS idx_project_views_recent 
ON project_views(project_id, viewed_at DESC);

-- Create a function to efficiently search projects
CREATE OR REPLACE FUNCTION search_projects(
    search_query TEXT,
    p_category project_category DEFAULT NULL,
    p_difficulty project_difficulty DEFAULT NULL,
    p_featured BOOLEAN DEFAULT NULL,
    p_limit INTEGER DEFAULT 20,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    user_id UUID,
    title VARCHAR(255),
    description TEXT,
    difficulty project_difficulty,
    category project_category,
    tags TEXT[],
    is_public BOOLEAN,
    featured BOOLEAN,
    view_count INTEGER,
    clone_count INTEGER,
    like_count INTEGER,
    comment_count INTEGER,
    created_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    author_name VARCHAR(255),
    author_avatar VARCHAR(500),
    organization_name VARCHAR(255),
    rank REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        p.id,
        p.user_id,
        p.title,
        p.description,
        p.difficulty,
        p.category,
        p.tags,
        p.is_public,
        p.featured,
        p.view_count,
        p.clone_count,
        p.like_count,
        p.comment_count,
        p.created_at,
        p.published_at,
        u.full_name AS author_name,
        u.avatar_url AS author_avatar,
        o.name AS organization_name,
        CASE 
            WHEN search_query IS NOT NULL AND search_query != '' 
            THEN ts_rank(p.search_vector, plainto_tsquery('english', search_query))
            ELSE 1.0
        END AS rank
    FROM user_projects p
    INNER JOIN user_profiles u ON p.user_id = u.id
    LEFT JOIN organizations o ON u.organization_id = o.id
    WHERE p.is_public = TRUE
        AND u.is_active = TRUE
        AND (search_query IS NULL OR search_query = '' OR p.search_vector @@ plainto_tsquery('english', search_query))
        AND (p_category IS NULL OR p.category = p_category)
        AND (p_difficulty IS NULL OR p.difficulty = p_difficulty)
        AND (p_featured IS NULL OR p.featured = p_featured)
    ORDER BY 
        CASE 
            WHEN search_query IS NOT NULL AND search_query != '' 
            THEN ts_rank(p.search_vector, plainto_tsquery('english', search_query))
            ELSE 0
        END DESC,
        p.featured DESC,
        p.like_count DESC,
        p.created_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql STABLE;

COMMENT ON FUNCTION search_projects IS 'Efficiently search public projects with full-text search, filters, and pagination';

-- Analyze tables for query optimization
ANALYZE organizations;
ANALYZE user_profiles;
ANALYZE user_projects;
ANALYZE project_likes;
ANALYZE project_comments;
ANALYZE project_views;
