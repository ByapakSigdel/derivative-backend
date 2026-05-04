//! Project handlers.

use axum::{
    extract::{Path, Query, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{CurrentUser, OptionalUser};
use crate::models::{
    CloneProjectRequest, CreateProjectRequest, ListProjectsQuery, ProjectResponse,
    ProjectStats, ProjectWithAuthorResponse, UpdateProjectRequest, UserProject,
};
use crate::services::{collaboration_service, project_service};
use crate::utils::pagination::PaginatedResponse;

/// List user's own projects
pub async fn list_projects(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Query(query): Query<ListProjectsQuery>,
) -> AppResult<Json<PaginatedResponse<ProjectResponse>>> {
    let projects = project_service::list_user_projects(&pool, user.id(), &query).await?;
    
    let response = PaginatedResponse {
        data: projects.data.into_iter().map(ProjectResponse::from).collect(),
        pagination: projects.pagination,
    };
    
    Ok(Json(response))
}

/// Create a new project
pub async fn create_project(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Json(request): Json<CreateProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
    request.validate()?;
    
    let project = project_service::create_project(&pool, user.id(), &request).await?;
    
    Ok(Json(ProjectResponse::from(project)))
}

/// Get user's project statistics
pub async fn get_stats(
    State(pool): State<DbPool>,
    user: CurrentUser,
) -> AppResult<Json<ProjectStats>> {
    let stats = project_service::get_user_stats(&pool, user.id()).await?;
    
    Ok(Json(stats))
}

/// List public projects (community)
pub async fn list_public_projects(
    State(pool): State<DbPool>,
    _user: OptionalUser,
    Query(query): Query<ListProjectsQuery>,
) -> AppResult<Json<PaginatedResponse<ProjectWithAuthorResponse>>> {
    let projects = project_service::list_public_projects(&pool, &query).await?;
    
    let response = PaginatedResponse {
        data: projects.data.into_iter().map(ProjectWithAuthorResponse::from).collect(),
        pagination: projects.pagination,
    };
    
    Ok(Json(response))
}

/// Get a single project
pub async fn get_project(
    State(pool): State<DbPool>,
    user: OptionalUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ProjectWithAuthorResponse>> {
    let project = project_service::get_project_with_author(&pool, id).await?;

    // Access rules for private projects:
    //   - the owner can always read
    //   - any user listed in project_collaborators can read
    // (The WebSocket handler uses can_user_access_project for exactly this
    // reason; align HTTP GET with that so a collaborator who just accepted an
    // invite can fetch the project content.)
    if !project.is_public {
        let allowed = match user.id() {
            Some(uid) if uid == project.user_id => true,
            Some(uid) => collaboration_service::can_user_access_project(&pool, id, uid)
                .await
                .unwrap_or(false),
            None => false,
        };
        if !allowed {
            return Err(AppError::NotFound("Project".to_string()));
        }
    }

    Ok(Json(ProjectWithAuthorResponse::from(project)))
}

/// Update a project
pub async fn update_project(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
    request.validate()?;
    
    let project = project_service::update_project(
        &pool,
        id,
        user.id(),
        &request,
        user.is_admin(),
    )
    .await?;
    
    Ok(Json(ProjectResponse::from(project)))
}

/// Delete a project
pub async fn delete_project(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<DeleteResponse>> {
    project_service::delete_project(&pool, id, user.id(), user.is_admin()).await?;
    
    Ok(Json(DeleteResponse {
        message: "Project deleted successfully".to_string(),
    }))
}

/// Clone a project
pub async fn clone_project(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<CloneProjectRequest>,
) -> AppResult<Json<ProjectResponse>> {
    request.validate()?;
    
    let project = project_service::clone_project(&pool, id, user.id(), &request).await?;
    
    Ok(Json(ProjectResponse::from(project)))
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    message: String,
}
