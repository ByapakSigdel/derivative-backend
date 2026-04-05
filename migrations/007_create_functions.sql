-- Migration: 007_create_functions
-- Description: Create PostgreSQL functions for business logic

-- Function: Check if a user is an admin
CREATE OR REPLACE FUNCTION is_admin(check_user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM user_profiles 
        WHERE id = check_user_id 
        AND user_type = 'admin' 
        AND is_active = TRUE
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function: Atomically increment project view count
CREATE OR REPLACE FUNCTION increment_project_views(p_project_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE user_projects 
    SET view_count = view_count + 1 
    WHERE id = p_project_id;
END;
$$ LANGUAGE plpgsql;

-- Function: Atomically increment project clone count
CREATE OR REPLACE FUNCTION increment_project_clones(p_project_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE user_projects 
    SET clone_count = clone_count + 1 
    WHERE id = p_project_id;
END;
$$ LANGUAGE plpgsql;

-- Function: Check if a user has liked a project
CREATE OR REPLACE FUNCTION user_liked_project(p_project_id UUID, p_user_id UUID)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM project_likes 
        WHERE project_id = p_project_id 
        AND user_id = p_user_id
    );
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function: Toggle like on a project (returns TRUE if liked, FALSE if unliked)
CREATE OR REPLACE FUNCTION toggle_project_like(p_project_id UUID, p_user_id UUID)
RETURNS BOOLEAN AS $$
DECLARE
    like_exists BOOLEAN;
BEGIN
    -- Check if like already exists
    SELECT EXISTS (
        SELECT 1 FROM project_likes 
        WHERE project_id = p_project_id 
        AND user_id = p_user_id
    ) INTO like_exists;
    
    IF like_exists THEN
        -- Remove the like
        DELETE FROM project_likes 
        WHERE project_id = p_project_id 
        AND user_id = p_user_id;
        RETURN FALSE;
    ELSE
        -- Add the like
        INSERT INTO project_likes (project_id, user_id) 
        VALUES (p_project_id, p_user_id);
        RETURN TRUE;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function: Recalculate all project counts (for maintenance/consistency)
CREATE OR REPLACE FUNCTION recalculate_project_counts()
RETURNS VOID AS $$
BEGIN
    -- Recalculate like counts
    UPDATE user_projects p
    SET like_count = (
        SELECT COUNT(*) FROM project_likes l WHERE l.project_id = p.id
    );
    
    -- Recalculate comment counts (excluding deleted)
    UPDATE user_projects p
    SET comment_count = (
        SELECT COUNT(*) FROM project_comments c 
        WHERE c.project_id = p.id AND c.is_deleted = FALSE
    );
    
    -- Recalculate view counts
    UPDATE user_projects p
    SET view_count = (
        SELECT COUNT(*) FROM project_views v WHERE v.project_id = p.id
    );
END;
$$ LANGUAGE plpgsql;

-- Function: Update search vector for a project
CREATE OR REPLACE FUNCTION update_project_search_vector()
RETURNS TRIGGER AS $$
BEGIN
    NEW.search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(NEW.description, '')), 'B') ||
        setweight(to_tsvector('english', COALESCE(array_to_string(NEW.tags, ' '), '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function: Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function: Set published_at when project becomes public
CREATE OR REPLACE FUNCTION set_published_at()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_public = TRUE AND OLD.is_public = FALSE AND NEW.published_at IS NULL THEN
        NEW.published_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function: Update like_count on likes table changes
CREATE OR REPLACE FUNCTION update_project_like_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        UPDATE user_projects 
        SET like_count = like_count + 1 
        WHERE id = NEW.project_id;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        UPDATE user_projects 
        SET like_count = like_count - 1 
        WHERE id = OLD.project_id;
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function: Update comment_count on comments table changes
CREATE OR REPLACE FUNCTION update_project_comment_count()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.is_deleted = FALSE THEN
            UPDATE user_projects 
            SET comment_count = comment_count + 1 
            WHERE id = NEW.project_id;
        END IF;
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        IF OLD.is_deleted = FALSE THEN
            UPDATE user_projects 
            SET comment_count = comment_count - 1 
            WHERE id = OLD.project_id;
        END IF;
        RETURN OLD;
    ELSIF TG_OP = 'UPDATE' THEN
        -- Handle soft delete
        IF OLD.is_deleted = FALSE AND NEW.is_deleted = TRUE THEN
            UPDATE user_projects 
            SET comment_count = comment_count - 1 
            WHERE id = NEW.project_id;
        ELSIF OLD.is_deleted = TRUE AND NEW.is_deleted = FALSE THEN
            UPDATE user_projects 
            SET comment_count = comment_count + 1 
            WHERE id = NEW.project_id;
        END IF;
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function: Set is_edited flag on comment update
CREATE OR REPLACE FUNCTION set_comment_edited_flag()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.content IS DISTINCT FROM NEW.content THEN
        NEW.is_edited = TRUE;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION is_admin IS 'Check if the given user ID belongs to an admin';
COMMENT ON FUNCTION increment_project_views IS 'Atomically increment view count for a project';
COMMENT ON FUNCTION increment_project_clones IS 'Atomically increment clone count for a project';
COMMENT ON FUNCTION user_liked_project IS 'Check if a user has liked a specific project';
COMMENT ON FUNCTION toggle_project_like IS 'Toggle like status for a user on a project';
COMMENT ON FUNCTION recalculate_project_counts IS 'Recalculate all denormalized counts for maintenance';
