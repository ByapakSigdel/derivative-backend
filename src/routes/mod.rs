//! Routes module for API endpoint definitions.

pub mod auth;
pub mod users;
pub mod projects;
pub mod community;
pub mod admin;
pub mod ws;

pub use auth::auth_routes;
pub use users::user_routes;
pub use projects::project_routes;
pub use community::community_routes;
pub use admin::admin_routes;
pub use ws::ws_routes;
