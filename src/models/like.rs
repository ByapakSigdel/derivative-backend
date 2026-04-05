//! Like model and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Project like entity from the database
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectLike {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Like status response
#[derive(Debug, Serialize)]
pub struct LikeStatusResponse {
    pub liked: bool,
    pub like_count: i32,
}

/// Toggle like response
#[derive(Debug, Serialize)]
pub struct ToggleLikeResponse {
    pub liked: bool,
    pub like_count: i32,
}

/// Like info for lists
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct LikeWithProject {
    pub id: Uuid,
    pub project_id: Uuid,
    pub project_title: String,
    pub created_at: DateTime<Utc>,
}

/// Query for listing user's likes
#[derive(Debug, Deserialize, Default)]
pub struct ListLikesQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}
