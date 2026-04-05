//! Project service for project management operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CloneProjectRequest, CreateProjectRequest, ListProjectsQuery, ProjectCategory,
    ProjectDifficulty, ProjectStats, ProjectWithAuthor, SortOrder, UpdateProjectRequest,
    UserProject,
};
use crate::utils::pagination::{PaginatedResponse, PaginationParams, Paginate};

/// Create a new project
pub async fn create_project(
    pool: &PgPool,
    user_id: Uuid,
    request: &CreateProjectRequest,
) -> AppResult<UserProject> {
    let project: UserProject = sqlx::query_as(
        r#"
        INSERT INTO user_projects (
            user_id, title, description, difficulty, category,
            nodes, edges, materials, learning_goals, tags, is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        RETURNING id, user_id, title, description, difficulty, category,
                  nodes, edges, materials, learning_goals, tags,
                  is_public, featured, view_count, clone_count, like_count, comment_count,
                  created_at, updated_at, published_at
        "#,
    )
    .bind(user_id)
    .bind(&request.title)
    .bind(request.description.as_deref())
    .bind(request.difficulty.unwrap_or_default())
    .bind(request.category.unwrap_or_default())
    .bind(request.nodes.clone().unwrap_or(serde_json::json!([])))
    .bind(request.edges.clone().unwrap_or(serde_json::json!([])))
    .bind(request.materials.clone().unwrap_or_default())
    .bind(request.learning_goals.clone().unwrap_or_default())
    .bind(request.tags.clone().unwrap_or_default())
    .bind(request.is_public.unwrap_or(false))
    .fetch_one(pool)
    .await?;
    
    Ok(project)
}

/// Get project by ID
pub async fn get_project_by_id(pool: &PgPool, project_id: Uuid) -> AppResult<UserProject> {
    sqlx::query_as(
        r#"
        SELECT id, user_id, title, description, difficulty, category,
               nodes, edges, materials, learning_goals, tags,
               is_public, featured, view_count, clone_count, like_count, comment_count,
               created_at, updated_at, published_at
        FROM user_projects
        WHERE id = $1
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Project".to_string()))
}

/// Get project with author info
pub async fn get_project_with_author(
    pool: &PgPool,
    project_id: Uuid,
) -> AppResult<ProjectWithAuthor> {
    sqlx::query_as(
        r#"
        SELECT p.id, p.user_id, p.title, p.description, p.difficulty, p.category,
               p.nodes, p.edges, p.materials, p.learning_goals, p.tags,
               p.is_public, p.featured, p.view_count, p.clone_count, p.like_count, p.comment_count,
               p.created_at, p.updated_at, p.published_at,
               u.id as author_id, u.full_name as author_name, u.email as author_email,
               u.avatar_url as author_avatar,
               o.id as organization_id, o.name as organization_name
        FROM user_projects p
        INNER JOIN user_profiles u ON p.user_id = u.id
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE p.id = $1
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Project".to_string()))
}

/// List user's own projects
pub async fn list_user_projects(
    pool: &PgPool,
    user_id: Uuid,
    query: &ListProjectsQuery,
) -> AppResult<PaginatedResponse<UserProject>> {
    let pagination = PaginationParams::new(query.page, query.per_page);
    
    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_projects
        WHERE user_id = $1
        AND ($2::project_category IS NULL OR category = $2)
        AND ($3::project_difficulty IS NULL OR difficulty = $3)
        AND ($4::text IS NULL OR search_vector @@ plainto_tsquery('english', $4))
        "#,
    )
    .bind(user_id)
    .bind(query.category)
    .bind(query.difficulty)
    .bind(query.search.as_deref())
    .fetch_one(pool)
    .await?;
    
    // Get projects
    let projects: Vec<UserProject> = sqlx::query_as(
        r#"
        SELECT id, user_id, title, description, difficulty, category,
               nodes, edges, materials, learning_goals, tags,
               is_public, featured, view_count, clone_count, like_count, comment_count,
               created_at, updated_at, published_at
        FROM user_projects
        WHERE user_id = $1
        AND ($2::project_category IS NULL OR category = $2)
        AND ($3::project_difficulty IS NULL OR difficulty = $3)
        AND ($4::text IS NULL OR search_vector @@ plainto_tsquery('english', $4))
        ORDER BY created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(user_id)
    .bind(query.category)
    .bind(query.difficulty)
    .bind(query.search.as_deref())
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;
    
    Ok(projects.paginate(&pagination, total.0))
}

/// List public projects (community)
pub async fn list_public_projects(
    pool: &PgPool,
    query: &ListProjectsQuery,
) -> AppResult<PaginatedResponse<ProjectWithAuthor>> {
    let pagination = PaginationParams::new(query.page, query.per_page);
    
    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_projects p
        INNER JOIN user_profiles u ON p.user_id = u.id
        WHERE p.is_public = TRUE AND u.is_active = TRUE
        AND ($1::project_category IS NULL OR p.category = $1)
        AND ($2::project_difficulty IS NULL OR p.difficulty = $2)
        AND ($3::bool IS NULL OR p.featured = $3)
        AND ($4::text IS NULL OR p.search_vector @@ plainto_tsquery('english', $4))
        "#,
    )
    .bind(query.category)
    .bind(query.difficulty)
    .bind(query.featured)
    .bind(query.search.as_deref())
    .fetch_one(pool)
    .await?;
    
    // Build sort clause
    let sort_column = match query.sort_by {
        Some(crate::models::ProjectSortBy::Title) => "p.title",
        Some(crate::models::ProjectSortBy::UpdatedAt) => "p.updated_at",
        Some(crate::models::ProjectSortBy::ViewCount) => "p.view_count",
        Some(crate::models::ProjectSortBy::LikeCount) => "p.like_count",
        Some(crate::models::ProjectSortBy::CloneCount) => "p.clone_count",
        Some(crate::models::ProjectSortBy::CommentCount) => "p.comment_count",
        _ => "p.created_at",
    };
    
    let sort_order = match query.sort_order {
        Some(SortOrder::Asc) => "ASC",
        _ => "DESC",
    };
    
    // Get projects with dynamic sort
    let sql = format!(
        r#"
        SELECT p.id, p.user_id, p.title, p.description, p.difficulty, p.category,
               p.nodes, p.edges, p.materials, p.learning_goals, p.tags,
               p.is_public, p.featured, p.view_count, p.clone_count, p.like_count, p.comment_count,
               p.created_at, p.updated_at, p.published_at,
               u.id as author_id, u.full_name as author_name, u.email as author_email,
               u.avatar_url as author_avatar,
               o.id as organization_id, o.name as organization_name
        FROM user_projects p
        INNER JOIN user_profiles u ON p.user_id = u.id
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE p.is_public = TRUE AND u.is_active = TRUE
        AND ($1::project_category IS NULL OR p.category = $1)
        AND ($2::project_difficulty IS NULL OR p.difficulty = $2)
        AND ($3::bool IS NULL OR p.featured = $3)
        AND ($4::text IS NULL OR p.search_vector @@ plainto_tsquery('english', $4))
        ORDER BY {} {}
        LIMIT $5 OFFSET $6
        "#,
        sort_column, sort_order
    );
    
    let projects: Vec<ProjectWithAuthor> = sqlx::query_as(&sql)
        .bind(query.category)
        .bind(query.difficulty)
        .bind(query.featured)
        .bind(query.search.as_deref())
        .bind(pagination.limit())
        .bind(pagination.offset())
        .fetch_all(pool)
        .await?;
    
    Ok(projects.paginate(&pagination, total.0))
}

/// Update a project
pub async fn update_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    request: &UpdateProjectRequest,
    is_admin: bool,
) -> AppResult<UserProject> {
    // Get existing project
    let existing = get_project_by_id(pool, project_id).await?;
    
    // Check ownership (admins can update any project)
    if existing.user_id != user_id && !is_admin {
        return Err(AppError::Forbidden);
    }
    
    // Only admins can set featured
    let featured = if is_admin {
        request.featured
    } else {
        None
    };
    
    let project: UserProject = sqlx::query_as(
        r#"
        UPDATE user_projects
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            difficulty = COALESCE($3, difficulty),
            category = COALESCE($4, category),
            nodes = COALESCE($5, nodes),
            edges = COALESCE($6, edges),
            materials = COALESCE($7, materials),
            learning_goals = COALESCE($8, learning_goals),
            tags = COALESCE($9, tags),
            is_public = COALESCE($10, is_public),
            featured = COALESCE($11, featured)
        WHERE id = $12
        RETURNING id, user_id, title, description, difficulty, category,
                  nodes, edges, materials, learning_goals, tags,
                  is_public, featured, view_count, clone_count, like_count, comment_count,
                  created_at, updated_at, published_at
        "#,
    )
    .bind(request.title.as_deref())
    .bind(request.description.as_deref())
    .bind(request.difficulty)
    .bind(request.category)
    .bind(request.nodes.as_ref())
    .bind(request.edges.as_ref())
    .bind(request.materials.as_ref())
    .bind(request.learning_goals.as_ref())
    .bind(request.tags.as_ref())
    .bind(request.is_public)
    .bind(featured)
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(project)
}

/// Delete a project
pub async fn delete_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    is_admin: bool,
) -> AppResult<()> {
    // Get existing project
    let existing = get_project_by_id(pool, project_id).await?;
    
    // Check ownership (admins can delete any project)
    if existing.user_id != user_id && !is_admin {
        return Err(AppError::Forbidden);
    }
    
    sqlx::query("DELETE FROM user_projects WHERE id = $1")
        .bind(project_id)
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Clone a project
pub async fn clone_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    request: &CloneProjectRequest,
) -> AppResult<UserProject> {
    // Get original project
    let original = get_project_by_id(pool, project_id).await?;
    
    // Check if project is public or owned by user
    if !original.is_public && original.user_id != user_id {
        return Err(AppError::Forbidden);
    }
    
    // Create cloned project
    let title = request
        .title
        .clone()
        .unwrap_or_else(|| format!("{} (Clone)", original.title));
    
    let cloned: UserProject = sqlx::query_as(
        r#"
        INSERT INTO user_projects (
            user_id, title, description, difficulty, category,
            nodes, edges, materials, learning_goals, tags, is_public
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, FALSE)
        RETURNING id, user_id, title, description, difficulty, category,
                  nodes, edges, materials, learning_goals, tags,
                  is_public, featured, view_count, clone_count, like_count, comment_count,
                  created_at, updated_at, published_at
        "#,
    )
    .bind(user_id)
    .bind(&title)
    .bind(&original.description)
    .bind(original.difficulty)
    .bind(original.category)
    .bind(&original.nodes)
    .bind(&original.edges)
    .bind(&original.materials)
    .bind(&original.learning_goals)
    .bind(&original.tags)
    .fetch_one(pool)
    .await?;
    
    // Increment clone count on original
    sqlx::query("SELECT increment_project_clones($1)")
        .bind(project_id)
        .execute(pool)
        .await?;
    
    Ok(cloned)
}

/// Get user's project statistics
pub async fn get_user_stats(pool: &PgPool, user_id: Uuid) -> AppResult<ProjectStats> {
    let stats: ProjectStats = sqlx::query_as(
        r#"
        SELECT 
            COUNT(*) as total_projects,
            COUNT(*) FILTER (WHERE is_public = TRUE) as public_projects,
            COUNT(*) FILTER (WHERE is_public = FALSE) as private_projects,
            COALESCE(SUM(view_count), 0) as total_views,
            COALESCE(SUM(like_count), 0) as total_likes,
            COALESCE(SUM(clone_count), 0) as total_clones,
            COALESCE(SUM(comment_count), 0) as total_comments
        FROM user_projects
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    
    Ok(stats)
}

/// Check if user can access a project
pub async fn can_access_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Option<Uuid>,
) -> AppResult<bool> {
    let project = get_project_by_id(pool, project_id).await?;
    
    // Public projects are accessible to everyone
    if project.is_public {
        return Ok(true);
    }
    
    // Private projects are only accessible to the owner
    match user_id {
        Some(uid) if uid == project.user_id => Ok(true),
        _ => Ok(false),
    }
}
