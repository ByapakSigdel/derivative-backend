//! Metrics and analytics routes.

use axum::{
    routing::{get, post},
    Router,
};

use crate::db::DbPool;
use crate::handlers::metrics as handlers;

/// Create admin metrics routes (protected by admin middleware in main.rs)
pub fn admin_metrics_routes() -> Router<DbPool> {
    Router::new()
        .route("/dashboard", get(handlers::get_dashboard_metrics))
        .route("/timeseries", get(handlers::get_metrics_time_series))
        .route(
            "/top-projects/views",
            get(handlers::get_top_projects_by_views),
        )
        .route(
            "/top-projects/likes",
            get(handlers::get_top_projects_by_likes),
        )
        .route("/categories", get(handlers::get_category_stats))
        .route("/difficulty", get(handlers::get_difficulty_stats))
        .route("/top-users", get(handlers::get_top_users))
        .route("/update-daily", post(handlers::update_daily_metrics))
}

/// Create user-accessible metrics routes (for logging)
pub fn metrics_routes() -> Router<DbPool> {
    Router::new()
        .route("/compilation", post(handlers::log_compilation))
        .route("/upload", post(handlers::log_upload))
}
