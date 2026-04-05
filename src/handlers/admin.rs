//! Admin handlers.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::AppResult;
use crate::middleware::auth::CurrentUser;
use crate::models::{
    CreateUserRequest, ListUsersQuery, UpdateUserRequest, UserResponse, UserWithOrgResponse,
};
use crate::services::user_service;
use crate::utils::pagination::PaginatedResponse;

/// List all users (admin only)
pub async fn list_users(
    State(pool): State<DbPool>,
    _user: CurrentUser, // Admin check is done by middleware
    Query(query): Query<ListUsersQuery>,
) -> AppResult<Json<PaginatedResponse<UserWithOrgResponse>>> {
    let users = user_service::list_users(&pool, &query).await?;
    
    let response = PaginatedResponse {
        data: users.data.into_iter().map(UserWithOrgResponse::from).collect(),
        pagination: users.pagination,
    };
    
    Ok(Json(response))
}

/// Create a new user (admin only)
pub async fn create_user(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Json(request): Json<CreateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    request.validate()?;
    
    let user = user_service::create_user(&pool, &request).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// Update a user (admin only)
pub async fn update_user(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> AppResult<Json<UserResponse>> {
    request.validate()?;
    
    let user = user_service::update_user(&pool, id, &request).await?;
    
    Ok(Json(UserResponse::from(user)))
}

/// Delete a user (admin only)
pub async fn delete_user(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeleteResponse>> {
    user_service::delete_user(&pool, id).await?;
    
    Ok(Json(DeleteResponse {
        message: "User deleted successfully".to_string(),
    }))
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    message: String,
}
