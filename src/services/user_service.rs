//! User service for user management operations.

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateUserRequest, ListUsersQuery, UpdateUserRequest, UserProfile, UserType,
    UserWithOrganization,
};
use crate::utils::pagination::{PaginatedResponse, PaginationParams, Paginate};
use crate::utils::password::{hash_password, validate_password_strength};

/// Create a new user
pub async fn create_user(pool: &PgPool, request: &CreateUserRequest) -> AppResult<UserProfile> {
    // Validate password strength
    validate_password_strength(&request.password)?;
    
    // Check if email already exists
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM user_profiles WHERE email = $1",
    )
    .bind(&request.email)
    .fetch_optional(pool)
    .await?;
    
    if exists.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }
    
    // Hash password
    let password_hash = hash_password(&request.password)?;
    
    // Insert user
    let user: UserProfile = sqlx::query_as(
        r#"
        INSERT INTO user_profiles (email, full_name, password_hash, user_type, organization_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, full_name, password_hash, user_type, organization_id,
                  avatar_url, is_active, refresh_token, refresh_token_expires_at,
                  created_at, updated_at
        "#,
    )
    .bind(&request.email)
    .bind(&request.full_name)
    .bind(&password_hash)
    .bind(request.user_type.unwrap_or(UserType::User))
    .bind(request.organization_id)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

/// Get user by ID
pub async fn get_user_by_id(pool: &PgPool, user_id: Uuid) -> AppResult<UserProfile> {
    sqlx::query_as(
        r#"
        SELECT id, email, full_name, password_hash, user_type, organization_id,
               avatar_url, is_active, refresh_token, refresh_token_expires_at,
               created_at, updated_at
        FROM user_profiles
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("User".to_string()))
}

/// Get user by email
pub async fn get_user_by_email(pool: &PgPool, email: &str) -> AppResult<Option<UserProfile>> {
    sqlx::query_as(
        r#"
        SELECT id, email, full_name, password_hash, user_type, organization_id,
               avatar_url, is_active, refresh_token, refresh_token_expires_at,
               created_at, updated_at
        FROM user_profiles
        WHERE email = $1
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)
}

/// List all users with pagination and filters (admin only)
pub async fn list_users(
    pool: &PgPool,
    query: &ListUsersQuery,
) -> AppResult<PaginatedResponse<UserWithOrganization>> {
    let pagination = PaginationParams::new(query.page, query.per_page);
    
    // Build dynamic query
    let mut sql = String::from(
        r#"
        SELECT u.id, u.email, u.full_name, u.user_type, u.organization_id,
               o.name as organization_name, u.avatar_url, u.is_active,
               u.created_at, u.updated_at
        FROM user_profiles u
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE 1=1
        "#,
    );
    
    let mut count_sql = String::from(
        "SELECT COUNT(*) FROM user_profiles u WHERE 1=1",
    );
    
    // Add filters
    let mut param_idx = 1;
    let mut bindings: Vec<String> = Vec::new();
    
    if let Some(ref search) = query.search {
        sql.push_str(&format!(
            " AND (u.email ILIKE ${} OR u.full_name ILIKE ${})",
            param_idx, param_idx
        ));
        count_sql.push_str(&format!(
            " AND (u.email ILIKE ${} OR u.full_name ILIKE ${})",
            param_idx, param_idx
        ));
        bindings.push(format!("%{}%", search));
        param_idx += 1;
    }
    
    if query.user_type.is_some() {
        sql.push_str(&format!(" AND u.user_type = ${}", param_idx));
        count_sql.push_str(&format!(" AND u.user_type = ${}", param_idx));
        param_idx += 1;
    }
    
    if query.organization_id.is_some() {
        sql.push_str(&format!(" AND u.organization_id = ${}", param_idx));
        count_sql.push_str(&format!(" AND u.organization_id = ${}", param_idx));
        param_idx += 1;
    }
    
    if query.is_active.is_some() {
        sql.push_str(&format!(" AND u.is_active = ${}", param_idx));
        count_sql.push_str(&format!(" AND u.is_active = ${}", param_idx));
        param_idx += 1;
    }
    
    sql.push_str(&format!(
        " ORDER BY u.created_at DESC LIMIT ${} OFFSET ${}",
        param_idx,
        param_idx + 1
    ));
    
    // For now, use a simpler approach with direct query binding
    // In production, consider using a query builder like sea-query
    
    // Get total count
    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*) FROM user_profiles u
        WHERE ($1::text IS NULL OR u.email ILIKE '%' || $1 || '%' OR u.full_name ILIKE '%' || $1 || '%')
        AND ($2::user_type IS NULL OR u.user_type = $2)
        AND ($3::uuid IS NULL OR u.organization_id = $3)
        AND ($4::bool IS NULL OR u.is_active = $4)
        "#,
    )
    .bind(query.search.as_deref())
    .bind(query.user_type)
    .bind(query.organization_id)
    .bind(query.is_active)
    .fetch_one(pool)
    .await?;
    
    // Get users
    let users: Vec<UserWithOrganization> = sqlx::query_as(
        r#"
        SELECT u.id, u.email, u.full_name, u.user_type, u.organization_id,
               o.name as organization_name, u.avatar_url, u.is_active,
               u.created_at, u.updated_at
        FROM user_profiles u
        LEFT JOIN organizations o ON u.organization_id = o.id
        WHERE ($1::text IS NULL OR u.email ILIKE '%' || $1 || '%' OR u.full_name ILIKE '%' || $1 || '%')
        AND ($2::user_type IS NULL OR u.user_type = $2)
        AND ($3::uuid IS NULL OR u.organization_id = $3)
        AND ($4::bool IS NULL OR u.is_active = $4)
        ORDER BY u.created_at DESC
        LIMIT $5 OFFSET $6
        "#,
    )
    .bind(query.search.as_deref())
    .bind(query.user_type)
    .bind(query.organization_id)
    .bind(query.is_active)
    .bind(pagination.limit())
    .bind(pagination.offset())
    .fetch_all(pool)
    .await?;
    
    Ok(users.paginate(&pagination, total.0))
}

/// Update user profile
pub async fn update_user(
    pool: &PgPool,
    user_id: Uuid,
    request: &UpdateUserRequest,
) -> AppResult<UserProfile> {
    // Check if user exists
    let existing = get_user_by_id(pool, user_id).await?;
    
    // If email is being changed, check for uniqueness
    if let Some(ref new_email) = request.email {
        if new_email != &existing.email {
            let exists: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM user_profiles WHERE email = $1 AND id != $2",
            )
            .bind(new_email)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
            
            if exists.is_some() {
                return Err(AppError::Conflict("Email already in use".to_string()));
            }
        }
    }
    
    // Hash password if provided
    let password_hash = if let Some(ref password) = request.password {
        validate_password_strength(password)?;
        Some(hash_password(password)?)
    } else {
        None
    };
    
    // Update user
    let user: UserProfile = sqlx::query_as(
        r#"
        UPDATE user_profiles
        SET email = COALESCE($1, email),
            full_name = COALESCE($2, full_name),
            password_hash = COALESCE($3, password_hash),
            user_type = COALESCE($4, user_type),
            organization_id = COALESCE($5, organization_id),
            avatar_url = COALESCE($6, avatar_url),
            is_active = COALESCE($7, is_active)
        WHERE id = $8
        RETURNING id, email, full_name, password_hash, user_type, organization_id,
                  avatar_url, is_active, refresh_token, refresh_token_expires_at,
                  created_at, updated_at
        "#,
    )
    .bind(request.email.as_deref())
    .bind(request.full_name.as_deref())
    .bind(password_hash.as_deref())
    .bind(request.user_type)
    .bind(request.organization_id)
    .bind(request.avatar_url.as_deref())
    .bind(request.is_active)
    .bind(user_id)
    .fetch_one(pool)
    .await?;
    
    Ok(user)
}

/// Update user avatar URL
pub async fn update_avatar(pool: &PgPool, user_id: Uuid, avatar_url: &str) -> AppResult<()> {
    sqlx::query(
        "UPDATE user_profiles SET avatar_url = $1 WHERE id = $2",
    )
    .bind(avatar_url)
    .bind(user_id)
    .execute(pool)
    .await?;
    
    Ok(())
}

/// Delete user
pub async fn delete_user(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM user_profiles WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User".to_string()));
    }
    
    Ok(())
}

/// Check if user is admin
pub async fn is_admin(pool: &PgPool, user_id: Uuid) -> AppResult<bool> {
    let result: Option<(UserType,)> = sqlx::query_as(
        "SELECT user_type FROM user_profiles WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    
    Ok(result.map(|(t,)| t == UserType::Admin).unwrap_or(false))
}
