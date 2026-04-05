//! Comment model and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Project comment entity from the database
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectComment {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub is_deleted: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Comment with author information
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CommentWithAuthor {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub is_deleted: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_name: String,
    pub author_avatar: Option<String>,
}

/// Request body for creating a comment
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommentRequest {
    #[validate(length(min = 1, max = 10000, message = "Comment must be between 1 and 10000 characters"))]
    pub content: String,
    
    pub parent_id: Option<Uuid>,
}

/// Request body for updating a comment
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCommentRequest {
    #[validate(length(min = 1, max = 10000, message = "Comment must be between 1 and 10000 characters"))]
    pub content: String,
}

/// Comment response for API
#[derive(Debug, Serialize)]
pub struct CommentResponse {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub content: String,
    pub is_deleted: bool,
    pub is_edited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: CommentAuthor,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub replies: Vec<CommentResponse>,
}

/// Author info for comments
#[derive(Debug, Serialize)]
pub struct CommentAuthor {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
}

impl From<CommentWithAuthor> for CommentResponse {
    fn from(comment: CommentWithAuthor) -> Self {
        let content = if comment.is_deleted {
            "[Comment deleted]".to_string()
        } else {
            comment.content
        };
        
        Self {
            id: comment.id,
            project_id: comment.project_id,
            user_id: comment.user_id,
            parent_id: comment.parent_id,
            content,
            is_deleted: comment.is_deleted,
            is_edited: comment.is_edited,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            author: CommentAuthor {
                id: comment.user_id,
                name: comment.author_name,
                avatar_url: comment.author_avatar,
            },
            replies: Vec::new(),
        }
    }
}

/// Query parameters for listing comments
#[derive(Debug, Deserialize, Default)]
pub struct ListCommentsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub parent_id: Option<Uuid>,
}

/// Threaded comment response (for nested display)
#[derive(Debug, Serialize)]
pub struct ThreadedCommentsResponse {
    pub comments: Vec<CommentResponse>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}
