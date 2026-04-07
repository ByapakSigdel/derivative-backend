//! Project routes.

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::projects as handlers;

/// Create project routes
pub fn project_routes() -> Router<DbPool> {
    Router::new()
        // User's own projects
        .route(
            "/",
            get(handlers::list_projects).post(handlers::create_project),
        )
        .route("/stats", get(handlers::get_stats))
        .route("/public", get(handlers::list_public_projects))
        .route("/:id", get(handlers::get_project))
        .route("/:id", patch(handlers::update_project))
        .route("/:id", delete(handlers::delete_project))
        .route("/:id/clone", post(handlers::clone_project))
}
