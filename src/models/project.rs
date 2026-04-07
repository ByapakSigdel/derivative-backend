//! Project model and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// Project difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "project_difficulty", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProjectDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

impl Default for ProjectDifficulty {
    fn default() -> Self {
        Self::Beginner
    }
}

/// Project category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "project_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ProjectCategory {
    Tutorial,
    Game,
    Simulation,
    Art,
    Music,
    Utility,
    Education,
    Other,
}

impl Default for ProjectCategory {
    fn default() -> Self {
        Self::Other
    }
}

/// User project entity from the database
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProject {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub difficulty: ProjectDifficulty,
    pub category: ProjectCategory,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub materials: Vec<String>,
    pub learning_goals: Vec<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub featured: bool,
    pub view_count: i32,
    pub clone_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

/// Project with author information (for public listing)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectWithAuthor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub difficulty: ProjectDifficulty,
    pub category: ProjectCategory,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub materials: Vec<String>,
    pub learning_goals: Vec<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub featured: bool,
    pub view_count: i32,
    pub clone_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_email: String,
    pub author_avatar: Option<String>,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
}

/// Request body for creating a project
#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: String,

    #[validate(length(max = 5000, message = "Description must be at most 5000 characters"))]
    pub description: Option<String>,

    pub difficulty: Option<ProjectDifficulty>,

    pub category: Option<ProjectCategory>,

    pub nodes: Option<serde_json::Value>,

    pub edges: Option<serde_json::Value>,

    pub materials: Option<Vec<String>>,

    pub learning_goals: Option<Vec<String>>,

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    pub tags: Option<Vec<String>>,

    pub is_public: Option<bool>,
}

/// Request body for updating a project
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateProjectRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: Option<String>,

    #[validate(length(max = 5000, message = "Description must be at most 5000 characters"))]
    pub description: Option<String>,

    pub difficulty: Option<ProjectDifficulty>,

    pub category: Option<ProjectCategory>,

    pub nodes: Option<serde_json::Value>,

    pub edges: Option<serde_json::Value>,

    pub materials: Option<Vec<String>>,

    pub learning_goals: Option<Vec<String>>,

    #[validate(length(max = 20, message = "Maximum 20 tags allowed"))]
    pub tags: Option<Vec<String>>,

    pub is_public: Option<bool>,

    pub featured: Option<bool>,
}

/// Project response for API
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub difficulty: ProjectDifficulty,
    pub category: ProjectCategory,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub materials: Vec<String>,
    pub learning_goals: Vec<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub featured: bool,
    pub view_count: i32,
    pub clone_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

impl From<UserProject> for ProjectResponse {
    fn from(project: UserProject) -> Self {
        Self {
            id: project.id,
            user_id: project.user_id,
            title: project.title,
            description: project.description,
            difficulty: project.difficulty,
            category: project.category,
            nodes: project.nodes,
            edges: project.edges,
            materials: project.materials,
            learning_goals: project.learning_goals,
            tags: project.tags,
            is_public: project.is_public,
            featured: project.featured,
            view_count: project.view_count,
            clone_count: project.clone_count,
            like_count: project.like_count,
            comment_count: project.comment_count,
            created_at: project.created_at,
            updated_at: project.updated_at,
            published_at: project.published_at,
        }
    }
}

/// Project response with author info
#[derive(Debug, Serialize)]
pub struct ProjectWithAuthorResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub difficulty: ProjectDifficulty,
    pub category: ProjectCategory,
    pub nodes: serde_json::Value,
    pub edges: serde_json::Value,
    pub materials: Vec<String>,
    pub learning_goals: Vec<String>,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub featured: bool,
    pub view_count: i32,
    pub clone_count: i32,
    pub like_count: i32,
    pub comment_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: AuthorInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_liked: Option<bool>,
}

/// Author information embedded in project response
#[derive(Debug, Serialize)]
pub struct AuthorInfo {
    pub id: Uuid,
    pub name: String,
    pub avatar_url: Option<String>,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
}

impl From<ProjectWithAuthor> for ProjectWithAuthorResponse {
    fn from(project: ProjectWithAuthor) -> Self {
        Self {
            id: project.id,
            user_id: project.user_id,
            title: project.title,
            description: project.description,
            difficulty: project.difficulty,
            category: project.category,
            nodes: project.nodes,
            edges: project.edges,
            materials: project.materials,
            learning_goals: project.learning_goals,
            tags: project.tags,
            is_public: project.is_public,
            featured: project.featured,
            view_count: project.view_count,
            clone_count: project.clone_count,
            like_count: project.like_count,
            comment_count: project.comment_count,
            created_at: project.created_at,
            updated_at: project.updated_at,
            published_at: project.published_at,
            author: AuthorInfo {
                id: project.author_id,
                name: project.author_name,
                avatar_url: project.author_avatar,
                organization_id: project.organization_id,
                organization_name: project.organization_name,
            },
            user_liked: None,
        }
    }
}

/// Query parameters for listing projects
#[derive(Debug, Deserialize, Default)]
pub struct ListProjectsQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub category: Option<ProjectCategory>,
    pub difficulty: Option<ProjectDifficulty>,
    pub featured: Option<bool>,
    pub sort_by: Option<ProjectSortBy>,
    pub sort_order: Option<SortOrder>,
}

/// Sort options for projects
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ProjectSortBy {
    #[default]
    CreatedAt,
    UpdatedAt,
    Title,
    ViewCount,
    LikeCount,
    CloneCount,
    CommentCount,
}

/// Sort order
#[derive(Debug, Clone, Copy, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    #[default]
    Desc,
}

/// Project statistics for a user
#[derive(Debug, Serialize, FromRow)]
pub struct ProjectStats {
    pub total_projects: i64,
    pub public_projects: i64,
    pub private_projects: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_clones: i64,
    pub total_comments: i64,
}

/// Clone project request
#[derive(Debug, Deserialize, Validate)]
pub struct CloneProjectRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "Title must be between 1 and 255 characters"
    ))]
    pub title: Option<String>,
}
