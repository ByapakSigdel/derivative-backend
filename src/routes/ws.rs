//! WebSocket routes.

use axum::{routing::get, Router};

use crate::db::DbPool;
use crate::handlers::ws as handlers;

/// Create WebSocket routes
pub fn ws_routes() -> Router<DbPool> {
    Router::new().route("/projects/:id", get(handlers::project_websocket))
}
