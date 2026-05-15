//! Admin dashboard handlers: cross-cutting views that don't fit neatly into
//! the existing per-resource modules.
//!
//! Everything here is mounted under /api/admin and gated by `require_admin`
//! in main.rs — no per-handler permission checks needed.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::websocket::handler::{room_manager, RoomSnapshot};

// ---------------- /api/admin/dashboard ----------------

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub users_total: i64,
    pub users_admins: i64,
    pub users_active: i64,
    pub projects_total: i64,
    pub projects_public: i64,
    pub contact_requests_total: i64,
    pub contact_requests_waiting: i64,
    pub online_users: usize,
    pub active_rooms: usize,
}

/// GET /api/admin/dashboard — one-shot aggregate snapshot for the admin
/// landing tab. Cheap because each query is a single COUNT(*).
pub async fn dashboard_stats(State(pool): State<DbPool>) -> AppResult<Json<DashboardStats>> {
    let (users_total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_profiles")
        .fetch_one(&pool)
        .await?;
    let (users_admins,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_profiles WHERE user_type = 'admin'")
            .fetch_one(&pool)
            .await?;
    let (users_active,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_profiles WHERE is_active = TRUE")
            .fetch_one(&pool)
            .await?;
    let (projects_total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM user_projects")
        .fetch_one(&pool)
        .await?;
    let (projects_public,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_projects WHERE is_public = TRUE")
            .fetch_one(&pool)
            .await?;
    let (contact_requests_total,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM contact_requests")
            .fetch_one(&pool)
            .await?;
    let (contact_requests_waiting,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM contact_requests WHERE contacted = FALSE")
            .fetch_one(&pool)
            .await?;

    // Live counts come from the in-process RoomManager (DashMap snapshot).
    let snapshot = room_manager().snapshot();

    Ok(Json(DashboardStats {
        users_total,
        users_admins,
        users_active,
        projects_total,
        projects_public,
        contact_requests_total,
        contact_requests_waiting,
        online_users: snapshot.total_users,
        active_rooms: snapshot.rooms.len(),
    }))
}

// ---------------- /api/admin/projects ----------------

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AdminProjectRow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_public: bool,
    pub view_count: i32,
    pub like_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_id: Uuid,
    pub owner_email: String,
    pub owner_full_name: Option<String>,
    pub owner_user_type: String,
    /// Number of nodes/edges, derived from JSON length so admin can spot
    /// "empty" projects without loading the full canvas blob.
    pub node_count: i32,
    pub edge_count: i32,
}

/// GET /api/admin/projects — every project on the platform, with owner
/// email + type. Used by the admin's Projects tab. Ordered newest-first.
pub async fn list_all_projects(
    State(pool): State<DbPool>,
) -> AppResult<Json<Vec<AdminProjectRow>>> {
    let rows: Vec<AdminProjectRow> = sqlx::query_as(
        r#"
        SELECT
            p.id, p.name, p.description, p.is_public, p.view_count, p.like_count,
            p.created_at, p.updated_at,
            u.id AS user_id, u.email AS owner_email, u.full_name AS owner_full_name,
            u.user_type AS owner_user_type,
            COALESCE(jsonb_array_length(p.nodes), 0) AS node_count,
            COALESCE(jsonb_array_length(p.edges), 0) AS edge_count
        FROM user_projects p
        INNER JOIN user_profiles u ON p.user_id = u.id
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

#[derive(Debug, Serialize)]
pub struct AdminDeleteResponse {
    pub deleted: bool,
}

/// DELETE /api/admin/projects/:id — admin can nuke any project regardless
/// of ownership. Returns `{ deleted: true }` on success, 404 if the row
/// wasn't there to begin with.
pub async fn delete_any_project(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AdminDeleteResponse>> {
    let result = sqlx::query("DELETE FROM user_projects WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("project".into()));
    }
    Ok(Json(AdminDeleteResponse { deleted: true }))
}

// ---------------- /api/admin/live ----------------

/// GET /api/admin/live — current WebSocket room state, served straight from
/// the in-process RoomManager. Read-only; doesn't take any DB locks.
pub async fn live_snapshot(State(_pool): State<DbPool>) -> AppResult<Json<RoomSnapshot>> {
    Ok(Json(room_manager().snapshot()))
}
