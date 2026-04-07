//! Metrics and analytics API handlers

use crate::errors::AppResult;
use crate::middleware::admin::AdminUser;
use crate::middleware::auth::CurrentUser;
use crate::models::{LogCompilationRequest, LogUploadRequest};
use crate::services::metrics_service;
use axum::{extract::{Query, State}, http::StatusCode, Json, response::IntoResponse};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct TimeSeriesQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct TopProjectsQuery {
    pub limit: Option<i32>,
}

/// Get dashboard overview metrics (admin only)
/// GET /api/admin/metrics/dashboard
pub async fn get_dashboard_metrics(
    State(pool): State<PgPool>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let metrics = metrics_service::get_dashboard_metrics(&pool).await?;
    Ok((StatusCode::OK, Json(metrics)))
}

/// Get metrics time series for charts (admin only)
/// GET /api/admin/metrics/timeseries?days=30
pub async fn get_metrics_time_series(
    State(pool): State<PgPool>,
    Query(query): Query<TimeSeriesQuery>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let days = query.days.unwrap_or(30);
    let metrics = metrics_service::get_metrics_time_series(&pool, days).await?;
    Ok((StatusCode::OK, Json(metrics)))
}

/// Get top projects by views (admin only)
/// GET /api/admin/metrics/top-projects/views?limit=10
pub async fn get_top_projects_by_views(
    State(pool): State<PgPool>,
    Query(query): Query<TopProjectsQuery>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let limit = query.limit.unwrap_or(10);
    let projects = metrics_service::get_top_projects_by_views(&pool, limit).await?;
    Ok((StatusCode::OK, Json(projects)))
}

/// Get top projects by likes (admin only)
/// GET /api/admin/metrics/top-projects/likes?limit=10
pub async fn get_top_projects_by_likes(
    State(pool): State<PgPool>,
    Query(query): Query<TopProjectsQuery>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let limit = query.limit.unwrap_or(10);
    let projects = metrics_service::get_top_projects_by_likes(&pool, limit).await?;
    Ok((StatusCode::OK, Json(projects)))
}

/// Get category distribution (admin only)
/// GET /api/admin/metrics/categories
pub async fn get_category_stats(
    State(pool): State<PgPool>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let stats = metrics_service::get_category_stats(&pool).await?;
    Ok((StatusCode::OK, Json(stats)))
}

/// Get difficulty distribution (admin only)
/// GET /api/admin/metrics/difficulty
pub async fn get_difficulty_stats(
    State(pool): State<PgPool>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let stats = metrics_service::get_difficulty_stats(&pool).await?;
    Ok((StatusCode::OK, Json(stats)))
}

/// Get most active users (admin only)
/// GET /api/admin/metrics/top-users?limit=10
pub async fn get_top_users(
    State(pool): State<PgPool>,
    Query(query): Query<TopProjectsQuery>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    let limit = query.limit.unwrap_or(10);
    let users = metrics_service::get_top_users(&pool, limit).await?;
    Ok((StatusCode::OK, Json(users)))
}

/// Log a compilation attempt
/// POST /api/metrics/compilation
pub async fn log_compilation(
    State(pool): State<PgPool>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<LogCompilationRequest>,
) -> AppResult<impl IntoResponse> {
    let log_id = metrics_service::log_compilation(&pool, user.id, req).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "log_id": log_id
    }))))
}

/// Log an upload attempt
/// POST /api/metrics/upload
pub async fn log_upload(
    State(pool): State<PgPool>,
    CurrentUser(user): CurrentUser,
    Json(req): Json<LogUploadRequest>,
) -> AppResult<impl IntoResponse> {
    let log_id = metrics_service::log_upload(&pool, user.id, req).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "log_id": log_id
    }))))
}

/// Manually update daily metrics (admin only)
/// POST /api/admin/metrics/update-daily
pub async fn update_daily_metrics(
    State(pool): State<PgPool>,
    AdminUser(_admin): AdminUser,
) -> AppResult<impl IntoResponse> {
    metrics_service::update_daily_metrics(&pool).await?;
    Ok((StatusCode::OK, Json(serde_json::json!({
        "message": "Daily metrics updated successfully"
    }))))
}
