//! Organization handlers for admin API.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::AppResult;
use crate::middleware::auth::CurrentUser;
use crate::models::{CreateOrganizationRequest, OrganizationResponse, UpdateOrganizationRequest};
use crate::services::organization_service;
use crate::utils::pagination::{PaginatedResponse, PaginationParams};

/// List all organizations (admin only)
pub async fn list_organizations(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<organization_service::OrganizationWithUserCount>>> {
    let organizations = organization_service::list_organizations_with_user_counts(&pool, &params).await?;
    Ok(Json(organizations))
}

/// Get a single organization (admin only)
pub async fn get_organization(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrganizationResponse>> {
    let organization = organization_service::get_organization(&pool, id).await?;
    Ok(Json(OrganizationResponse::from(organization)))
}

/// Create a new organization (admin only)
pub async fn create_organization(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Json(request): Json<CreateOrganizationRequest>,
) -> AppResult<Json<OrganizationResponse>> {
    request.validate()?;
    
    let organization = organization_service::create_organization(&pool, &request).await?;
    Ok(Json(OrganizationResponse::from(organization)))
}

/// Update an organization (admin only)
pub async fn update_organization(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateOrganizationRequest>,
) -> AppResult<Json<OrganizationResponse>> {
    request.validate()?;
    
    let organization = organization_service::update_organization(&pool, id, &request).await?;
    Ok(Json(OrganizationResponse::from(organization)))
}

/// Delete an organization (admin only)
pub async fn delete_organization(
    State(pool): State<DbPool>,
    _user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeleteResponse>> {
    organization_service::delete_organization(&pool, id).await?;
    
    Ok(Json(DeleteResponse {
        message: "Organization deleted successfully".to_string(),
    }))
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    message: String,
}
