//! Org Admin routes — mounted at `/api/org`. Require authentication; the
//! handlers enforce that the caller is an Org Admin and scope everything to
//! their own organization.

use axum::{routing::get, Router};

use crate::db::DbPool;
use crate::handlers::org_admin as h;

/// Routes mounted at `/api/org`.
pub fn org_routes() -> Router<DbPool> {
    Router::new()
        .route("/members", get(h::list_members).post(h::create_member))
        .route(
            "/members/:id",
            get(h::get_member)
                .patch(h::update_member)
                .delete(h::delete_member),
        )
}
