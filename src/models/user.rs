//! User model and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;
use validator::Validate;

/// User type enum matching PostgreSQL enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "user_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserType {
    Admin,
    User,
}

impl Default for UserType {
    fn default() -> Self {
        Self::User
    }
}

/// User profile entity from the database
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub user_type: UserType,
    pub organization_id: Option<Uuid>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    #[serde(skip_serializing)]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing)]
    pub refresh_token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User with organization info (for admin listing)
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserWithOrganization {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub user_type: UserType,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request body for user registration/creation
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Full name must be between 1 and 255 characters"
    ))]
    pub full_name: String,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: String,

    pub user_type: Option<UserType>,

    pub organization_id: Option<Uuid>,
}

/// Request body for updating a user
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: Option<String>,

    #[validate(length(
        min = 1,
        max = 255,
        message = "Full name must be between 1 and 255 characters"
    ))]
    pub full_name: Option<String>,

    #[validate(length(
        min = 8,
        max = 128,
        message = "Password must be between 8 and 128 characters"
    ))]
    pub password: Option<String>,

    pub user_type: Option<UserType>,

    pub organization_id: Option<Uuid>,

    pub avatar_url: Option<String>,

    pub is_active: Option<bool>,
}

/// Request body for login
#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

/// Request body for refreshing tokens
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

/// Response for authentication endpoints
#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

/// User response for API (without sensitive data)
#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub user_type: UserType,
    pub organization_id: Option<Uuid>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserProfile> for UserResponse {
    fn from(user: UserProfile) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            user_type: user.user_type,
            organization_id: user.organization_id,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// User response with organization details
#[derive(Debug, Clone, Serialize)]
pub struct UserWithOrgResponse {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub user_type: UserType,
    pub organization_id: Option<Uuid>,
    pub organization_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserWithOrganization> for UserWithOrgResponse {
    fn from(user: UserWithOrganization) -> Self {
        Self {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            user_type: user.user_type,
            organization_id: user.organization_id,
            organization_name: user.organization_name,
            avatar_url: user.avatar_url,
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

/// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub search: Option<String>,
    pub user_type: Option<UserType>,
    pub organization_id: Option<Uuid>,
    pub is_active: Option<bool>,
}
