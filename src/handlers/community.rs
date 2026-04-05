//! Community handlers (likes, comments, views).

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    Json,
};
use std::net::SocketAddr;
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::AppResult;
use crate::middleware::auth::{CurrentUser, OptionalUser};
use crate::models::{
    CommentResponse, CreateCommentRequest, LikeStatusResponse, ListCommentsQuery,
    RecordViewRequest, ThreadedCommentsResponse, ToggleLikeResponse, UpdateCommentRequest,
    ViewResponse,
};
use crate::services::community_service;

// ==================== LIKES ====================

/// Toggle like on a project
pub async fn toggle_like(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ToggleLikeResponse>> {
    let response = community_service::toggle_like(&pool, id, user.id()).await?;
    
    Ok(Json(response))
}

/// Get like status for a project
pub async fn get_like_status(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LikeStatusResponse>> {
    let response = community_service::get_like_status(&pool, id, user.id()).await?;
    
    Ok(Json(response))
}

/// Unlike a project
pub async fn unlike_project(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<LikeStatusResponse>> {
    let response = community_service::unlike_project(&pool, id, user.id()).await?;
    
    Ok(Json(response))
}

// ==================== VIEWS ====================

/// Record a project view
pub async fn record_view(
    State(pool): State<DbPool>,
    user: OptionalUser,
    Path(id): Path<Uuid>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Json(request): Json<RecordViewRequest>,
) -> AppResult<Json<ViewResponse>> {
    let response = community_service::record_view(
        &pool,
        id,
        user.id(),
        Some(addr.ip()),
        &request,
    )
    .await?;
    
    Ok(Json(response))
}

// ==================== COMMENTS ====================

/// Get comments for a project
pub async fn get_comments(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Query(query): Query<ListCommentsQuery>,
) -> AppResult<Json<ThreadedCommentsResponse>> {
    let response = community_service::get_comments(&pool, id, &query).await?;
    
    Ok(Json(response))
}

/// Create a comment on a project
pub async fn create_comment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateCommentRequest>,
) -> AppResult<Json<CommentResponse>> {
    request.validate()?;
    
    let response = community_service::create_comment(&pool, id, user.id(), &request).await?;
    
    Ok(Json(response))
}

/// Get a single comment
pub async fn get_comment(
    State(pool): State<DbPool>,
    Path((_project_id, comment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<CommentResponse>> {
    let response = community_service::get_comment(&pool, comment_id).await?;
    
    Ok(Json(response))
}

/// Update a comment
pub async fn update_comment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path((_project_id, comment_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateCommentRequest>,
) -> AppResult<Json<CommentResponse>> {
    request.validate()?;
    
    let response = community_service::update_comment(&pool, comment_id, user.id(), &request).await?;
    
    Ok(Json(response))
}

/// Delete a comment (soft delete)
pub async fn delete_comment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path((_project_id, comment_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<DeleteResponse>> {
    community_service::delete_comment(&pool, comment_id, user.id(), user.is_admin()).await?;
    
    Ok(Json(DeleteResponse {
        message: "Comment deleted successfully".to_string(),
    }))
}

#[derive(serde::Serialize)]
pub struct DeleteResponse {
    message: String,
}
