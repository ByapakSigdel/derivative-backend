//! Admin routes.

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::admin as handlers;

/// Create admin routes
pub fn admin_routes() -> Router<DbPool> {
    Router::new()
        .route("/users", get(handlers::list_users))
        .route("/users", post(handlers::create_user))
        .route("/users/:id", patch(handlers::update_user))
        .route("/users/:id", delete(handlers::delete_user))
}
