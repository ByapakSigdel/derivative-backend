//! Collaboration API handlers

use crate::errors::AppResult;
use crate::middleware::auth::CurrentUser;
use crate::models::{AcceptInviteRequest, CreateInviteTokenRequest};
use crate::services::collaboration_service;
use axum::{extract::{Path, State}, http::StatusCode, Json, response::IntoResponse};
use sqlx::PgPool;
use uuid::Uuid;

/// Create an invite token for a project
/// POST /api/projects/:id/invites
pub async fn create_invite_token(
    State(pool): State<PgPool>,
    Path(project_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<CreateInviteTokenRequest>,
) -> AppResult<impl IntoResponse> {
    let invite = collaboration_service::create_invite_token(
        &pool,
        project_id,
        user.id,
        req,
    )
    .await?;

    Ok((StatusCode::OK, Json(invite)))
}

/// List all active invite tokens for a project
/// GET /api/projects/:id/invites
pub async fn list_invite_tokens(
    State(pool): State<PgPool>,
    Path(project_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let tokens = collaboration_service::list_invite_tokens(
        &pool,
        project_id,
        user.id,
    )
    .await?;

    Ok((StatusCode::OK, Json(tokens)))
}

/// Deactivate an invite token
/// DELETE /api/projects/invites/:token_id
pub async fn deactivate_invite_token(
    State(pool): State<PgPool>,
    Path(token_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    collaboration_service::deactivate_invite_token(&pool, token_id, user.id).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "Invite token deactivated successfully"
    }))))
}

/// Accept an invite token and join project
/// POST /api/projects/join
pub async fn accept_invite(
    State(pool): State<PgPool>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<AcceptInviteRequest>,
) -> AppResult<impl IntoResponse> {
    let collaborator =
        collaboration_service::accept_invite_token(&pool, user.id, &req.token).await?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "Successfully joined project",
        "project_id": collaborator.project_id,
        "role": collaborator.role
    }))))
}

/// List all collaborators for a project
/// GET /api/projects/:id/collaborators
pub async fn list_collaborators(
    State(pool): State<PgPool>,
    Path(project_id): Path<Uuid>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let response =
        collaboration_service::list_collaborators(&pool, project_id, user.id).await?;

    Ok((StatusCode::OK, Json(response)))
}

/// Remove a collaborator from a project
/// DELETE /api/projects/:id/collaborators/:user_id
pub async fn remove_collaborator(
    State(pool): State<PgPool>,
    Path((project_id, collaborator_user_id)): Path<(Uuid, Uuid)>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    collaboration_service::remove_collaborator(
        &pool,
        project_id,
        collaborator_user_id,
        user.id,
    )
    .await?;

    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "Collaborator removed successfully"
    }))))
}
