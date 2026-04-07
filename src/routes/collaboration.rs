//! Collaboration routes.

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::collaboration as handlers;

/// Create collaboration routes
pub fn collaboration_routes() -> Router<DbPool> {
    Router::new()
        // Invite tokens
        .route("/projects/:id/invites", post(handlers::create_invite_token))
        .route("/projects/:id/invites", get(handlers::list_invite_tokens))
        .route(
            "/projects/invites/:token_id",
            delete(handlers::deactivate_invite_token),
        )
        // Join project
        .route("/projects/join", post(handlers::accept_invite))
        // Collaborators management
        .route(
            "/projects/:id/collaborators",
            get(handlers::list_collaborators),
        )
        .route(
            "/projects/:id/collaborators/:user_id",
            delete(handlers::remove_collaborator),
        )
}
