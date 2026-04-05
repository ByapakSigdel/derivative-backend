//! Community service for likes, comments, and views.

use sqlx::PgPool;
use std::net::IpAddr;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CommentResponse, CommentWithAuthor, CreateCommentRequest, LikeStatusResponse,
    ListCommentsQuery, ProjectComment, RecordViewRequest, ThreadedCommentsResponse,
    ToggleLikeResponse, UpdateCommentRequest, ViewResponse,
};
use crate::utils::pagination::PaginationParams;

// ==================== LIKES ====================

/// Toggle like on a project
pub async fn toggle_like(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<ToggleLikeResponse> {
    // Check if project exists and is accessible
    let project_exists: Option<(bool,)> = sqlx::query_as(
        "SELECT is_public FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    
    if project_exists.is_none() {
        return Err(AppError::NotFound("Project".to_string()));
    }
    
    // Use the database function to toggle like
    let liked: (bool,) = sqlx::query_as("SELECT toggle_project_like($1, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    
    // Get updated like count
    let like_count: (i32,) = sqlx::query_as(
        "SELECT like_count FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(ToggleLikeResponse {
        liked: liked.0,
        like_count: like_count.0,
    })
}

/// Check if user liked a project
pub async fn get_like_status(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<LikeStatusResponse> {
    let liked: (bool,) = sqlx::query_as("SELECT user_liked_project($1, $2)")
        .bind(project_id)
        .bind(user_id)
        .fetch_one(pool)
        .await?;
    
    let like_count: (i32,) = sqlx::query_as(
        "SELECT like_count FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(LikeStatusResponse {
        liked: liked.0,
        like_count: like_count.0,
    })
}

/// Unlike a project (explicit removal)
pub async fn unlike_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<LikeStatusResponse> {
    sqlx::query(
        "DELETE FROM project_likes WHERE project_id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(user_id)
    .execute(pool)
    .await?;
    
    let like_count: (i32,) = sqlx::query_as(
        "SELECT like_count FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(LikeStatusResponse {
        liked: false,
        like_count: like_count.0,
    })
}

// ==================== VIEWS ====================

/// Record a project view
pub async fn record_view(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Option<Uuid>,
    ip_address: Option<IpAddr>,
    request: &RecordViewRequest,
) -> AppResult<ViewResponse> {
    // Check if project exists
    let project_exists: Option<(i32,)> = sqlx::query_as(
        "SELECT view_count FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    
    if project_exists.is_none() {
        return Err(AppError::NotFound("Project".to_string()));
    }
    
    // Insert view record
    sqlx::query(
        r#"
        INSERT INTO project_views (project_id, user_id, view_duration, referrer, ip_address)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .bind(request.view_duration)
    .bind(request.referrer.as_deref())
    .bind(ip_address)
    .execute(pool)
    .await?;
    
    // Increment view count
    sqlx::query("SELECT increment_project_views($1)")
        .bind(project_id)
        .execute(pool)
        .await?;
    
    // Get updated view count
    let view_count: (i32,) = sqlx::query_as(
        "SELECT view_count FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    Ok(ViewResponse {
        view_count: view_count.0,
        recorded: true,
    })
}

// ==================== COMMENTS ====================

/// Create a comment on a project
pub async fn create_comment(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    request: &CreateCommentRequest,
) -> AppResult<CommentResponse> {
    // Check if project exists
    let project_exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM user_projects WHERE id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    
    if project_exists.is_none() {
        return Err(AppError::NotFound("Project".to_string()));
    }
    
    // If parent_id is provided, verify it exists and belongs to this project
    if let Some(parent_id) = request.parent_id {
        let parent_exists: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM project_comments WHERE id = $1 AND project_id = $2",
        )
        .bind(parent_id)
        .bind(project_id)
        .fetch_optional(pool)
        .await?;
        
        if parent_exists.is_none() {
            return Err(AppError::NotFound("Parent comment".to_string()));
        }
    }
    
    // Insert comment
    let comment: CommentWithAuthor = sqlx::query_as(
        r#"
        INSERT INTO project_comments (project_id, user_id, parent_id, content)
        VALUES ($1, $2, $3, $4)
        RETURNING 
            id, project_id, user_id, parent_id, content, is_deleted, is_edited,
            created_at, updated_at,
            (SELECT full_name FROM user_profiles WHERE id = $2) as author_name,
            (SELECT avatar_url FROM user_profiles WHERE id = $2) as author_avatar
        "#,
    )
    .bind(project_id)
    .bind(user_id)
    .bind(request.parent_id)
    .bind(&request.content)
    .fetch_one(pool)
    .await?;
    
    Ok(CommentResponse::from(comment))
}

/// Get comments for a project (threaded)
pub async fn get_comments(
    pool: &PgPool,
    project_id: Uuid,
    query: &ListCommentsQuery,
) -> AppResult<ThreadedCommentsResponse> {
    let pagination = PaginationParams::new(query.page, query.per_page);
    
    // Get total count of top-level comments
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM project_comments
        WHERE project_id = $1 AND parent_id IS NULL AND is_deleted = FALSE
        "#,
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;
    
    // Get top-level comments
    let top_level: Vec<CommentWithAuthor> = sqlx::query_as(
        r#"
        SELECT c.id, c.project_id, c.user_id, c.parent_id, c.content,
               c.is_deleted, c.is_edited, c.created_at, c.updated_at,
               u.full_name as author_name, u.avatar_url as author_avatar
        FROM project_comments c
        INNER JOIN user_profiles u ON c.user_id = u.id
        WHERE c.project_id = $1 AND c.parent_id IS NULL
        ORDER BY c.created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(project_id)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;
    
    // Get all replies for these top-level comments
    let top_level_ids: Vec<Uuid> = top_level.iter().map(|c| c.id).collect();
    
    let replies: Vec<CommentWithAuthor> = if !top_level_ids.is_empty() {
        sqlx::query_as(
            r#"
            SELECT c.id, c.project_id, c.user_id, c.parent_id, c.content,
                   c.is_deleted, c.is_edited, c.created_at, c.updated_at,
                   u.full_name as author_name, u.avatar_url as author_avatar
            FROM project_comments c
            INNER JOIN user_profiles u ON c.user_id = u.id
            WHERE c.parent_id = ANY($1)
            ORDER BY c.created_at ASC
            "#,
        )
        .bind(&top_level_ids)
        .fetch_all(pool)
        .await?
    } else {
        Vec::new()
    };
    
    // Build threaded response
    let mut comments: Vec<CommentResponse> = top_level
        .into_iter()
        .map(CommentResponse::from)
        .collect();
    
    // Attach replies to their parent comments
    for comment in &mut comments {
        comment.replies = replies
            .iter()
            .filter(|r| r.parent_id == Some(comment.id))
            .cloned()
            .map(CommentResponse::from)
            .collect();
    }
    
    let total_pages = ((total.0 as f64) / (pagination.per_page as f64)).ceil() as u32;
    
    Ok(ThreadedCommentsResponse {
        comments,
        total: total.0,
        page: pagination.page,
        per_page: pagination.per_page,
        total_pages: total_pages.max(1),
    })
}

/// Get a single comment
pub async fn get_comment(pool: &PgPool, comment_id: Uuid) -> AppResult<CommentResponse> {
    let comment: CommentWithAuthor = sqlx::query_as(
        r#"
        SELECT c.id, c.project_id, c.user_id, c.parent_id, c.content,
               c.is_deleted, c.is_edited, c.created_at, c.updated_at,
               u.full_name as author_name, u.avatar_url as author_avatar
        FROM project_comments c
        INNER JOIN user_profiles u ON c.user_id = u.id
        WHERE c.id = $1
        "#,
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Comment".to_string()))?;
    
    Ok(CommentResponse::from(comment))
}

/// Update a comment (owner only)
pub async fn update_comment(
    pool: &PgPool,
    comment_id: Uuid,
    user_id: Uuid,
    request: &UpdateCommentRequest,
) -> AppResult<CommentResponse> {
    // Get existing comment
    let existing: ProjectComment = sqlx::query_as(
        r#"
        SELECT id, project_id, user_id, parent_id, content, is_deleted, is_edited,
               created_at, updated_at
        FROM project_comments
        WHERE id = $1
        "#,
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Comment".to_string()))?;
    
    // Check ownership
    if existing.user_id != user_id {
        return Err(AppError::Forbidden);
    }
    
    // Can't edit deleted comments
    if existing.is_deleted {
        return Err(AppError::BadRequest("Cannot edit deleted comment".to_string()));
    }
    
    // Update comment (is_edited flag is set by trigger)
    let comment: CommentWithAuthor = sqlx::query_as(
        r#"
        UPDATE project_comments
        SET content = $1
        WHERE id = $2
        RETURNING 
            id, project_id, user_id, parent_id, content, is_deleted, is_edited,
            created_at, updated_at,
            (SELECT full_name FROM user_profiles WHERE id = user_id) as author_name,
            (SELECT avatar_url FROM user_profiles WHERE id = user_id) as author_avatar
        "#,
    )
    .bind(&request.content)
    .bind(comment_id)
    .fetch_one(pool)
    .await?;
    
    Ok(CommentResponse::from(comment))
}

/// Soft delete a comment (owner only)
pub async fn delete_comment(
    pool: &PgPool,
    comment_id: Uuid,
    user_id: Uuid,
    is_admin: bool,
) -> AppResult<()> {
    // Get existing comment
    let existing: ProjectComment = sqlx::query_as(
        r#"
        SELECT id, project_id, user_id, parent_id, content, is_deleted, is_edited,
               created_at, updated_at
        FROM project_comments
        WHERE id = $1
        "#,
    )
    .bind(comment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Comment".to_string()))?;
    
    // Check ownership (admins can delete any comment)
    if existing.user_id != user_id && !is_admin {
        return Err(AppError::Forbidden);
    }
    
    // Soft delete
    sqlx::query(
        "UPDATE project_comments SET is_deleted = TRUE WHERE id = $1",
    )
    .bind(comment_id)
    .execute(pool)
    .await?;
    
    Ok(())
}
