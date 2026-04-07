//! Organization service for business logic.

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    CreateOrganizationRequest, Organization, OrganizationResponse, UpdateOrganizationRequest,
};
use crate::utils::pagination::{PaginatedResponse, PaginationMeta, PaginationParams};

/// List all organizations with pagination
pub async fn list_organizations(
    pool: &PgPool,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<OrganizationResponse>> {
    let params = params.normalize();
    
    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations")
        .fetch_one(pool)
        .await?;

    // Get organizations
    let organizations: Vec<Organization> = sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM organizations
        ORDER BY name ASC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;

    Ok(PaginatedResponse {
        data: organizations.into_iter().map(OrganizationResponse::from).collect(),
        pagination: PaginationMeta::new(&params, total),
    })
}

/// Get a single organization by ID
pub async fn get_organization(pool: &PgPool, id: Uuid) -> AppResult<Organization> {
    let organization: Option<Organization> = sqlx::query_as(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM organizations
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    organization.ok_or_else(|| AppError::NotFound("Organization not found".to_string()))
}

/// Create a new organization
pub async fn create_organization(
    pool: &PgPool,
    request: &CreateOrganizationRequest,
) -> AppResult<Organization> {
    // Check if organization with same name exists
    let existing: Option<Organization> = sqlx::query_as(
        "SELECT id, name, description, created_at, updated_at FROM organizations WHERE LOWER(name) = LOWER($1)",
    )
    .bind(&request.name)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "Organization with this name already exists".to_string(),
        ));
    }

    let organization: Organization = sqlx::query_as(
        r#"
        INSERT INTO organizations (name, description)
        VALUES ($1, $2)
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(&request.name)
    .bind(&request.description)
    .fetch_one(pool)
    .await?;

    Ok(organization)
}

/// Update an organization
pub async fn update_organization(
    pool: &PgPool,
    id: Uuid,
    request: &UpdateOrganizationRequest,
) -> AppResult<Organization> {
    // Check if organization exists
    let existing = get_organization(pool, id).await?;

    // If name is being changed, check for conflicts
    if let Some(ref new_name) = request.name {
        if new_name.to_lowercase() != existing.name.to_lowercase() {
            let conflict: Option<Organization> = sqlx::query_as(
                "SELECT id, name, description, created_at, updated_at FROM organizations WHERE LOWER(name) = LOWER($1) AND id != $2",
            )
            .bind(new_name)
            .bind(id)
            .fetch_optional(pool)
            .await?;

            if conflict.is_some() {
                return Err(AppError::Conflict(
                    "Organization with this name already exists".to_string(),
                ));
            }
        }
    }

    let name = request.name.as_ref().unwrap_or(&existing.name);
    let description = request.description.clone().or(existing.description);

    let organization: Organization = sqlx::query_as(
        r#"
        UPDATE organizations
        SET name = $2, description = $3, updated_at = NOW()
        WHERE id = $1
        RETURNING id, name, description, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(organization)
}

/// Delete an organization
pub async fn delete_organization(pool: &PgPool, id: Uuid) -> AppResult<()> {
    // Check if organization exists
    let _ = get_organization(pool, id).await?;

    // Check if any users are in this organization
    let user_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_profiles WHERE organization_id = $1",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    if user_count > 0 {
        return Err(AppError::Conflict(format!(
            "Cannot delete organization with {} users. Reassign users first.",
            user_count
        )));
    }

    sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Organization with user count for listing
#[derive(Debug, serde::Serialize)]
pub struct OrganizationWithUserCount {
    #[serde(flatten)]
    pub organization: OrganizationResponse,
    pub user_count: i64,
}

/// List organizations with user counts
pub async fn list_organizations_with_user_counts(
    pool: &PgPool,
    params: &PaginationParams,
) -> AppResult<PaginatedResponse<OrganizationWithUserCount>> {
    let params = params.normalize();

    // Get total count
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM organizations")
        .fetch_one(pool)
        .await?;

    // Get organizations with user counts
    let rows: Vec<(Uuid, String, Option<String>, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>, i64)> = sqlx::query_as(
        r#"
        SELECT 
            o.id, 
            o.name, 
            o.description, 
            o.created_at, 
            o.updated_at,
            COUNT(u.id)::bigint as user_count
        FROM organizations o
        LEFT JOIN user_profiles u ON u.organization_id = o.id
        GROUP BY o.id
        ORDER BY o.name ASC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(params.limit())
    .bind(params.offset())
    .fetch_all(pool)
    .await?;

    let data = rows
        .into_iter()
        .map(|(id, name, description, created_at, updated_at, user_count)| {
            OrganizationWithUserCount {
                organization: OrganizationResponse {
                    id,
                    name,
                    description,
                    created_at,
                    updated_at,
                },
                user_count,
            }
        })
        .collect();

    Ok(PaginatedResponse {
        data,
        pagination: PaginationMeta::new(&params, total),
    })
}
