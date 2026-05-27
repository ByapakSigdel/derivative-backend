//! Org Admin handlers — managing the teachers/students of one organization.
//!
//! Every endpoint is scoped to the caller's own organization (taken from their
//! token, never the request body). Platform admins manage org admins and any
//! user through the existing `/api/admin/users` endpoints instead.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::CurrentUser;
use crate::models::{
    CreateOrgMemberRequest, ListOrgMembersQuery, UpdateOrgMemberRequest, UserResponse,
    UserWithOrgResponse,
};
use crate::services::user_service;
use crate::utils::pagination::PaginatedResponse;

/// Require the caller to be an Org Admin and return the organization they
/// administer. Org admins always carry an organization_id in their token.
fn org_scope(user: &CurrentUser) -> AppResult<Uuid> {
    if !user.auth().is_org_admin() {
        return Err(AppError::Forbidden);
    }
    user.organization_id().ok_or(AppError::Forbidden)
}

/// List teachers/students in the caller's organization.
pub async fn list_members(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Query(query): Query<ListOrgMembersQuery>,
) -> AppResult<Json<PaginatedResponse<UserWithOrgResponse>>> {
    let org_id = org_scope(&user)?;
    let members = user_service::list_org_members(&pool, org_id, &query).await?;
    let response = PaginatedResponse {
        data: members
            .data
            .into_iter()
            .map(UserWithOrgResponse::from)
            .collect(),
        pagination: members.pagination,
    };
    Ok(Json(response))
}

/// Create a teacher/student in the caller's organization.
pub async fn create_member(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Json(request): Json<CreateOrgMemberRequest>,
) -> AppResult<Json<UserResponse>> {
    let org_id = org_scope(&user)?;
    request.validate()?;
    let created = user_service::create_org_member(&pool, org_id, &request).await?;
    Ok(Json(UserResponse::from(created)))
}

/// Fetch one member of the caller's organization.
pub async fn get_member(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<UserResponse>> {
    let org_id = org_scope(&user)?;
    let member = user_service::get_org_member(&pool, org_id, id).await?;
    Ok(Json(UserResponse::from(member)))
}

/// Update a member of the caller's organization.
pub async fn update_member(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateOrgMemberRequest>,
) -> AppResult<Json<UserResponse>> {
    let org_id = org_scope(&user)?;
    request.validate()?;
    let member = user_service::update_org_member(&pool, org_id, id, &request).await?;
    Ok(Json(UserResponse::from(member)))
}

/// Remove a member from the caller's organization.
pub async fn delete_member(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeleteResponse>> {
    let org_id = org_scope(&user)?;
    user_service::delete_org_member(&pool, org_id, id).await?;
    Ok(Json(DeleteResponse {
        message: "Member removed successfully".to_string(),
    }))
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    message: String,
}
