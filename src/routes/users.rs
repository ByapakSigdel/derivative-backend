//! User routes.

use axum::{routing::post, Router};

use crate::db::DbPool;
use crate::handlers::users as handlers;

/// Create user routes
pub fn user_routes() -> Router<DbPool> {
    Router::new()
        .route("/avatar", post(handlers::upload_avatar))
}
