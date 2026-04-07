//! Admin-only middleware for protected routes.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};

use crate::errors::AppError;
use crate::middleware::auth::AuthUser;
use crate::models::UserType;

/// Middleware to require admin privileges
/// Must be used AFTER the require_auth middleware
pub async fn require_admin(req: Request, next: Next) -> Result<Response, AppError> {
    // Get auth user from extensions (set by require_auth middleware)
    let auth_user = req
        .extensions()
        .get::<AuthUser>()
        .ok_or(AppError::Unauthorized)?;
    
    // Check if user is admin
    if auth_user.user_type != UserType::Admin {
        return Err(AppError::Forbidden);
    }
    
    Ok(next.run(req).await)
}

/// Helper to check admin status in handlers
pub fn ensure_admin(auth_user: &AuthUser) -> Result<(), AppError> {
    if auth_user.user_type != UserType::Admin {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

/// Admin user extractor for handlers (verifies admin status)
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthUser);

impl AdminUser {
    pub fn user(&self) -> &AuthUser {
        &self.0
    }
}

/// Axum extractor for admin user
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AdminUser
where
    S: Send + Sync,
{
    type Rejection = AppError;
    
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_user = parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(AppError::Unauthorized)?;
        
        // Verify admin status
        if auth_user.user_type != UserType::Admin {
            return Err(AppError::Forbidden);
        }
        
        Ok(AdminUser(auth_user))
    }
}
