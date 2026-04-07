//! Community routes (likes, comments, views).

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::community as handlers;

/// Create community routes (nested under /api/user-projects/:id)
pub fn community_routes() -> Router<DbPool> {
    Router::new()
        // Likes
        .route("/:id/like", post(handlers::toggle_like))
        .route("/:id/like", get(handlers::get_like_status))
        .route("/:id/like", delete(handlers::unlike_project))
        // Views
        .route("/:id/view", post(handlers::record_view))
        // Comments
        .route("/:id/comments", get(handlers::get_comments))
        .route("/:id/comments", post(handlers::create_comment))
        .route("/:id/comments/:comment_id", get(handlers::get_comment))
        .route("/:id/comments/:comment_id", patch(handlers::update_comment))
        .route(
            "/:id/comments/:comment_id",
            delete(handlers::delete_comment),
        )
}
