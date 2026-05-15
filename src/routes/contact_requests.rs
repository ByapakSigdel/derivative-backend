//! Contact request routes.
//!
//! Two router functions:
//!   - `public_contact_request_routes` — POST only, mounted with no auth.
//!   - `admin_contact_request_routes`  — GET list + PATCH, mounted under
//!     the admin auth layer in main.rs.

use axum::{
    routing::{get, patch, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::contact_requests as handlers;

/// Public-facing routes: anyone can submit a contact request.
pub fn public_contact_request_routes() -> Router<DbPool> {
    Router::new().route("/", post(handlers::create_contact_request))
}

/// Admin-only routes: list submissions and mark them as contacted.
pub fn admin_contact_request_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(handlers::list_contact_requests))
        .route("/:id", patch(handlers::update_contact_request))
}
