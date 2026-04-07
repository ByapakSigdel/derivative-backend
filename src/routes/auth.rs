//! Authentication routes.

use axum::{
    routing::{get, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::auth as handlers;

/// Create public authentication routes (no auth required)
pub fn auth_routes() -> Router<DbPool> {
    Router::new()
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh))
}

/// Create protected authentication routes (auth required)
pub fn protected_auth_routes() -> Router<DbPool> {
    Router::new()
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
}
