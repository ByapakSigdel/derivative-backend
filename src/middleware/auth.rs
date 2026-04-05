//! Authentication middleware for JWT verification.

use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::models::UserType;
use crate::utils::jwt::{extract_bearer_token, verify_access_token, AccessTokenClaims};

/// Extension type for authenticated user information
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub email: String,
    pub user_type: UserType,
}

impl From<AccessTokenClaims> for AuthUser {
    fn from(claims: AccessTokenClaims) -> Self {
        Self {
            id: claims.sub,
            email: claims.email,
            user_type: claims.user_type,
        }
    }
}

impl AuthUser {
    /// Check if the user is an admin
    pub fn is_admin(&self) -> bool {
        self.user_type == UserType::Admin
    }
}

/// Middleware to require authentication
pub async fn require_auth(
    State(_pool): State<DbPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Extract Authorization header
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(AppError::Unauthorized)?;
    
    // Extract bearer token
    let token = extract_bearer_token(auth_header).ok_or(AppError::Unauthorized)?;
    
    // Verify token and extract claims
    let claims = verify_access_token(token)?;
    
    // Create AuthUser and add to request extensions
    let auth_user = AuthUser::from(claims);
    req.extensions_mut().insert(auth_user);
    
    Ok(next.run(req).await)
}

/// Middleware to optionally extract auth (for routes that work both ways)
pub async fn optional_auth(
    State(_pool): State<DbPool>,
    mut req: Request,
    next: Next,
) -> Response {
    // Try to extract and verify token, but don't fail if not present
    if let Some(auth_header) = req.headers().get(AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = extract_bearer_token(auth_header) {
            if let Ok(claims) = verify_access_token(token) {
                let auth_user = AuthUser::from(claims);
                req.extensions_mut().insert(auth_user);
            }
        }
    }
    
    next.run(req).await
}

/// Extract AuthUser from request extensions
pub fn get_auth_user(req: &Request) -> Option<&AuthUser> {
    req.extensions().get::<AuthUser>()
}

/// Helper to extract current user from extensions (for handlers)
#[derive(Debug, Clone)]
pub struct CurrentUser(pub AuthUser);

impl CurrentUser {
    pub fn id(&self) -> Uuid {
        self.0.id
    }
    
    pub fn email(&self) -> &str {
        &self.0.email
    }
    
    pub fn user_type(&self) -> UserType {
        self.0.user_type
    }
    
    pub fn is_admin(&self) -> bool {
        self.0.is_admin()
    }
}

/// Axum extractor for current user
#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = AppError;
    
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .map(CurrentUser)
            .ok_or(AppError::Unauthorized)
    }
}

/// Optional current user extractor
#[derive(Debug, Clone)]
pub struct OptionalUser(pub Option<AuthUser>);

impl OptionalUser {
    pub fn id(&self) -> Option<Uuid> {
        self.0.as_ref().map(|u| u.id)
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.0.is_some()
    }
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for OptionalUser
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;
    
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(OptionalUser(parts.extensions.get::<AuthUser>().cloned()))
    }
}
