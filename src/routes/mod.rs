//! Routes module for API endpoint definitions.

pub mod admin;
pub mod auth;
pub mod collaboration;
pub mod community;
pub mod metrics;
pub mod projects;
pub mod users;
pub mod ws;

pub use admin::admin_routes;
pub use auth::{auth_routes, protected_auth_routes};
pub use collaboration::collaboration_routes;
pub use community::community_routes;
pub use metrics::{admin_metrics_routes, metrics_routes};
pub use projects::project_routes;
pub use users::user_routes;
pub use ws::ws_routes;
