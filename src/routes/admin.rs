//! Admin routes.

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::admin as admin_handlers;
use crate::handlers::organizations as org_handlers;

/// Create admin routes
pub fn admin_routes() -> Router<DbPool> {
    Router::new()
        // User management
        .route("/users", get(admin_handlers::list_users))
        .route("/users", post(admin_handlers::create_user))
        .route("/users/:id", patch(admin_handlers::update_user))
        .route("/users/:id", delete(admin_handlers::delete_user))
        // Organization management
        .route("/organizations", get(org_handlers::list_organizations))
        .route("/organizations", post(org_handlers::create_organization))
        .route("/organizations/:id", get(org_handlers::get_organization))
        .route(
            "/organizations/:id",
            patch(org_handlers::update_organization),
        )
        .route(
            "/organizations/:id",
            delete(org_handlers::delete_organization),
        )
}
