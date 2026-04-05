//! Authentication handlers.

use axum::{extract::State, Json};
use serde::Serialize;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::CurrentUser;
use crate::models::{AuthResponse, LoginRequest, RefreshTokenRequest, UserResponse};
use crate::services::auth_service;

/// Login with email and password
pub async fn login(
    State(pool): State<DbPool>,
    Json(request): Json<LoginRequest>,
) -> AppResult<Json<AuthResponse>> {
    request.validate()?;
    
    let response = auth_service::login(&pool, &request).await?;
    
    Ok(Json(response))
}

/// Logout and invalidate refresh token
pub async fn logout(
    State(pool): State<DbPool>,
    user: CurrentUser,
) -> AppResult<Json<LogoutResponse>> {
    auth_service::logout(&pool, user.id()).await?;
    
    Ok(Json(LogoutResponse {
        message: "Logged out successfully".to_string(),
    }))
}

#[derive(Serialize)]
pub struct LogoutResponse {
    message: String,
}

/// Refresh access token
pub async fn refresh(
    State(pool): State<DbPool>,
    Json(request): Json<RefreshTokenRequest>,
) -> AppResult<Json<AuthResponse>> {
    let response = auth_service::refresh_tokens(&pool, &request.refresh_token).await?;
    
    Ok(Json(response))
}

/// Get current user profile
pub async fn me(
    State(pool): State<DbPool>,
    user: CurrentUser,
) -> AppResult<Json<UserResponse>> {
    let profile = auth_service::get_current_user(&pool, user.id()).await?;
    
    Ok(Json(UserResponse::from(profile)))
}
