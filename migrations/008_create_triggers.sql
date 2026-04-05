-- Migration: 008_create_triggers
-- Description: Create triggers for automatic data management

-- Trigger: Update search_vector on project title/description change
CREATE TRIGGER trigger_update_project_search_vector
    BEFORE INSERT OR UPDATE OF title, description, tags ON user_projects
    FOR EACH ROW
    EXECUTE FUNCTION update_project_search_vector();

-- Trigger: Auto-update updated_at for organizations
CREATE TRIGGER trigger_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger: Auto-update updated_at for user_profiles
CREATE TRIGGER trigger_user_profiles_updated_at
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger: Auto-update updated_at for user_projects
CREATE TRIGGER trigger_user_projects_updated_at
    BEFORE UPDATE ON user_projects
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger: Auto-update updated_at for project_comments
CREATE TRIGGER trigger_project_comments_updated_at
    BEFORE UPDATE ON project_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger: Set published_at when project becomes public
CREATE TRIGGER trigger_set_published_at
    BEFORE UPDATE ON user_projects
    FOR EACH ROW
    WHEN (NEW.is_public = TRUE AND OLD.is_public = FALSE)
    EXECUTE FUNCTION set_published_at();

-- Trigger: Update like_count on likes insert
CREATE TRIGGER trigger_project_likes_insert
    AFTER INSERT ON project_likes
    FOR EACH ROW
    EXECUTE FUNCTION update_project_like_count();

-- Trigger: Update like_count on likes delete
CREATE TRIGGER trigger_project_likes_delete
    AFTER DELETE ON project_likes
    FOR EACH ROW
    EXECUTE FUNCTION update_project_like_count();

-- Trigger: Update comment_count on comments insert
CREATE TRIGGER trigger_project_comments_insert
    AFTER INSERT ON project_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_project_comment_count();

-- Trigger: Update comment_count on comments delete
CREATE TRIGGER trigger_project_comments_delete
    AFTER DELETE ON project_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_project_comment_count();

-- Trigger: Update comment_count on soft delete
CREATE TRIGGER trigger_project_comments_update
    AFTER UPDATE OF is_deleted ON project_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_project_comment_count();

-- Trigger: Set is_edited flag when comment content changes
CREATE TRIGGER trigger_comment_edited_flag
    BEFORE UPDATE OF content ON project_comments
    FOR EACH ROW
    EXECUTE FUNCTION set_comment_edited_flag();

COMMENT ON TRIGGER trigger_update_project_search_vector ON user_projects IS 'Automatically update full-text search vector when project title/description changes';
COMMENT ON TRIGGER trigger_set_published_at ON user_projects IS 'Set published_at timestamp when project first becomes public';
