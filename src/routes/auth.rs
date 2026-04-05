//! Authentication routes.

use axum::{
    routing::{get, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::auth as handlers;

/// Create authentication routes
pub fn auth_routes() -> Router<DbPool> {
    Router::new()
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/refresh", post(handlers::refresh))
        .route("/me", get(handlers::me))
}
