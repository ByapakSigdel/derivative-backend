//! Authentication service for user login, logout, and token management.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::{AppError, AppResult};
use crate::models::{AuthResponse, LoginRequest, UserProfile, UserResponse};
use crate::utils::jwt::{generate_access_token, generate_refresh_token, verify_refresh_token};
use crate::utils::password::verify_password;

/// Authenticate user with email and password
pub async fn login(pool: &PgPool, request: &LoginRequest) -> AppResult<AuthResponse> {
    info!("Login attempt for email: {}", request.email);
    
    // Find user by email
    let user: UserProfile = sqlx::query_as(
        r#"
        SELECT id, email, full_name, password_hash, user_type, organization_id,
               avatar_url, is_active, refresh_token, refresh_token_expires_at,
               created_at, updated_at
        FROM user_profiles
        WHERE email = $1
        "#,
    )
    .bind(&request.email)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| {
        warn!("Login failed: User not found for email: {}", request.email);
        AppError::InvalidCredentials
    })?;
    
    debug!("User found: id={}, email={}, is_active={}", user.id, user.email, user.is_active);
    
    // Check if account is active
    if !user.is_active {
        warn!("Login failed: Account is inactive for user: {}", user.email);
        return Err(AppError::Forbidden);
    }
    
    // Verify password
    match verify_password(&request.password, &user.password_hash) {
        Ok(true) => {
            debug!("Password verification successful for user: {}", user.email);
        }
        Ok(false) => {
            warn!("Login failed: Invalid password for user: {}", user.email);
            return Err(AppError::InvalidCredentials);
        }
        Err(e) => {
            error!("Password verification error for user {}: {:?}", user.email, e);
            return Err(e);
        }
    }
    
    // Generate tokens
    let access_token = generate_access_token(user.id, &user.email, user.user_type)?;
    let (refresh_token, _jti) = generate_refresh_token(user.id)?;
    
    // Calculate refresh token expiration
    let refresh_expires_at = Utc::now() + Duration::seconds(CONFIG.jwt_refresh_expiry);
    
    // Store refresh token in database
    sqlx::query(
        r#"
        UPDATE user_profiles
        SET refresh_token = $1, refresh_token_expires_at = $2
        WHERE id = $3
        "#,
    )
    .bind(&refresh_token)
    .bind(refresh_expires_at)
    .bind(user.id)
    .execute(pool)
    .await?;
    
    info!("Login successful for user: {} (id: {})", user.email, user.id);
    
    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: CONFIG.jwt_access_expiry,
        user: UserResponse::from(user),
    })
}

/// Refresh access token using refresh token
pub async fn refresh_tokens(pool: &PgPool, refresh_token: &str) -> AppResult<AuthResponse> {
    // Verify refresh token
    let claims = verify_refresh_token(refresh_token)?;
    
    // Find user and verify stored refresh token
    let user: UserProfile = sqlx::query_as(
        r#"
        SELECT id, email, full_name, password_hash, user_type, organization_id,
               avatar_url, is_active, refresh_token, refresh_token_expires_at,
               created_at, updated_at
        FROM user_profiles
        WHERE id = $1 AND refresh_token = $2 AND is_active = TRUE
        "#,
    )
    .bind(claims.sub)
    .bind(refresh_token)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::InvalidToken)?;
    
    // Check if refresh token is expired in database
    if let Some(expires_at) = user.refresh_token_expires_at {
        if expires_at < Utc::now() {
            return Err(AppError::TokenExpired);
        }
    }
    
    // Generate new tokens (token rotation)
    let new_access_token = generate_access_token(user.id, &user.email, user.user_type)?;
    let (new_refresh_token, _jti) = generate_refresh_token(user.id)?;
    
    // Calculate new refresh token expiration
    let refresh_expires_at = Utc::now() + Duration::seconds(CONFIG.jwt_refresh_expiry);
    
    // Update refresh token in database
    sqlx::query(
        r#"
        UPDATE user_profiles
        SET refresh_token = $1, refresh_token_expires_at = $2
        WHERE id = $3
        "#,
    )
    .bind(&new_refresh_token)
    .bind(refresh_expires_at)
    .bind(user.id)
    .execute(pool)
    .await?;
    
    Ok(AuthResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: CONFIG.jwt_access_expiry,
        user: UserResponse::from(user),
    })
}

/// Logout user by invalidating refresh token
pub async fn logout(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE user_profiles
        SET refresh_token = NULL, refresh_token_expires_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Get current user profile
pub async fn get_current_user(pool: &PgPool, user_id: Uuid) -> AppResult<UserProfile> {
    sqlx::query_as(
        r#"
        SELECT id, email, full_name, password_hash, user_type, organization_id,
               avatar_url, is_active, refresh_token, refresh_token_expires_at,
               created_at, updated_at
        FROM user_profiles
        WHERE id = $1 AND is_active = TRUE
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("User".to_string()))
}

/// Verify if user ID belongs to an active user
pub async fn verify_user_exists(pool: &PgPool, user_id: Uuid) -> AppResult<bool> {
    let result: Option<(bool,)> = sqlx::query_as(
        "SELECT is_active FROM user_profiles WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(result.map(|(active,)| active).unwrap_or(false))
}
