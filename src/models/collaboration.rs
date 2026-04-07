// ! Collaboration model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// Collaborator role in a project
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum CollaboratorRole {
    #[sqlx(rename = "owner")]
    Owner,
    #[sqlx(rename = "editor")]
    Editor,
    #[sqlx(rename = "viewer")]
    Viewer,
}

impl Default for CollaboratorRole {
    fn default() -> Self {
        Self::Editor
    }
}

/// Project collaborator entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectCollaborator {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub invited_by: Uuid,
    pub invited_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Project collaborator with user information
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CollaboratorWithUser {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub invited_by: Uuid,
    pub invited_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // User info
    pub user_email: String,
    pub user_name: Option<String>,
    pub user_avatar: Option<String>,
}

/// Project invite token entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectInviteToken {
    pub id: Uuid,
    pub project_id: Uuid,
    pub token: String,
    pub role: String,
    pub created_by: Uuid,
    pub max_uses: Option<i32>,
    pub uses_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Request to create an invite token
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateInviteTokenRequest {
    pub role: Option<String>, // Defaults to 'editor'
    #[validate(range(min = 1, max = 1000))]
    pub max_uses: Option<i32>, // NULL = unlimited
    pub expires_in_hours: Option<i32>, // NULL = never expires
}

/// Request to add a collaborator directly
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AddCollaboratorRequest {
    #[validate(email)]
    pub user_email: String,
    pub role: Option<String>, // Defaults to 'editor'
}

/// Request to accept an invite
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AcceptInviteRequest {
    #[validate(length(min = 1))]
    pub token: String,
}

/// Response with invite token details
#[derive(Debug, Clone, Serialize)]
pub struct InviteTokenResponse {
    pub id: Uuid,
    pub token: String,
    pub role: String,
    pub invite_url: String, // Full URL to accept invite
    pub max_uses: Option<i32>,
    pub uses_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// Response with list of collaborators
#[derive(Debug, Clone, Serialize)]
pub struct CollaboratorsResponse {
    pub collaborators: Vec<CollaboratorWithUser>,
    pub can_edit: bool, // Whether requesting user can edit collaborators
}
