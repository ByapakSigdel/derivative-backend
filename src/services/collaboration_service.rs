//! Collaboration service - manages project collaborators and invite tokens

use crate::errors::{AppError, AppResult};
use crate::models::{
    AddCollaboratorRequest, CollaboratorWithUser, CollaboratorsResponse, CreateInviteTokenRequest,
    InviteTokenResponse, ProjectCollaborator, ProjectInviteToken,
};
use chrono::{Duration, Utc};
use rand::Rng;
use sqlx::PgPool;
use uuid::Uuid;

/// Generate a random secure token for invites
fn generate_token() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    const TOKEN_LEN: usize = 32;
    let mut rng = rand::thread_rng();
    (0..TOKEN_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Create an invite token for a project
pub async fn create_invite_token(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
    req: CreateInviteTokenRequest,
) -> AppResult<InviteTokenResponse> {
    // Verify user owns or can edit the project
    let can_edit = can_user_edit_project(pool, project_id, user_id).await?;
    if !can_edit {
        return Err(AppError::Forbidden);
    }

    let token = generate_token();
    let role = req.role.unwrap_or_else(|| "editor".to_string());
    let expires_at = req
        .expires_in_hours
        .map(|hours| Utc::now() + Duration::hours(hours as i64));

    let invite = sqlx::query_as::<_, ProjectInviteToken>(
        r#"
        INSERT INTO project_invite_tokens 
        (project_id, token, role, created_by, max_uses, expires_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(project_id)
    .bind(&token)
    .bind(&role)
    .bind(user_id)
    .bind(req.max_uses)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(InviteTokenResponse {
        id: invite.id,
        token: invite.token.clone(),
        role: invite.role,
        invite_url: format!("/editor?join={}", invite.token),
        max_uses: invite.max_uses,
        uses_count: invite.uses_count,
        expires_at: invite.expires_at,
        created_at: invite.created_at,
        is_active: invite.is_active,
    })
}

/// Accept an invite token and join project as collaborator
pub async fn accept_invite_token(
    pool: &PgPool,
    user_id: Uuid,
    token: &str,
) -> AppResult<ProjectCollaborator> {
    // Get and validate token
    let invite = sqlx::query_as::<_, ProjectInviteToken>(
        r#"
        SELECT * FROM project_invite_tokens
        WHERE token = $1 AND is_active = TRUE
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Invite token not found or inactive".to_string()))?;

    // Check if token expired
    if let Some(expires_at) = invite.expires_at {
        if expires_at < Utc::now() {
            return Err(AppError::BadRequest(
                "Invite token has expired".to_string(),
            ));
        }
    }

    // Check if max uses exceeded
    if let Some(max_uses) = invite.max_uses {
        if invite.uses_count >= max_uses {
            return Err(AppError::BadRequest(
                "Invite token has reached maximum uses".to_string(),
            ));
        }
    }

    // Check if user is already a collaborator
    let existing = sqlx::query_as::<_, ProjectCollaborator>(
        "SELECT * FROM project_collaborators WHERE project_id = $1 AND user_id = $2",
    )
    .bind(invite.project_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest(
            "You are already a collaborator on this project".to_string(),
        ));
    }

    // Add user as collaborator
    let collaborator = sqlx::query_as::<_, ProjectCollaborator>(
        r#"
        INSERT INTO project_collaborators 
        (project_id, user_id, role, invited_by, accepted_at)
        VALUES ($1, $2, $3, $4, NOW())
        RETURNING *
        "#,
    )
    .bind(invite.project_id)
    .bind(user_id)
    .bind(&invite.role)
    .bind(invite.created_by)
    .fetch_one(pool)
    .await?;

    // Increment token usage count
    sqlx::query(
        "UPDATE project_invite_tokens SET uses_count = uses_count + 1 WHERE id = $1",
    )
    .bind(invite.id)
    .execute(pool)
    .await?;

    Ok(collaborator)
}

/// List all collaborators for a project
pub async fn list_collaborators(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<CollaboratorsResponse> {
    // Check if user can access project
    let can_access = can_user_access_project(pool, project_id, user_id).await?;
    if !can_access {
        return Err(AppError::Forbidden);
    }

    let can_edit = can_user_edit_project(pool, project_id, user_id).await?;

    let collaborators = sqlx::query_as::<_, CollaboratorWithUser>(
        r#"
        SELECT 
            c.*,
            u.email as user_email,
            u.full_name as user_name,
            u.avatar_url as user_avatar
        FROM project_collaborators c
        JOIN user_profiles u ON u.id = c.user_id
        WHERE c.project_id = $1 AND c.accepted_at IS NOT NULL
        ORDER BY c.created_at ASC
        "#,
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(CollaboratorsResponse {
        collaborators,
        can_edit,
    })
}

/// Remove a collaborator from a project
pub async fn remove_collaborator(
    pool: &PgPool,
    project_id: Uuid,
    collaborator_user_id: Uuid,
    requesting_user_id: Uuid,
) -> AppResult<()> {
    // Check if requesting user can edit project
    let can_edit = can_user_edit_project(pool, project_id, requesting_user_id).await?;
    if !can_edit {
        return Err(AppError::Forbidden);
    }

    // Don't allow removing the project owner
    let is_owner = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM user_projects WHERE id = $1 AND user_id = $2)",
    )
    .bind(project_id)
    .bind(collaborator_user_id)
    .fetch_one(pool)
    .await?;

    if is_owner {
        return Err(AppError::BadRequest(
            "Cannot remove project owner".to_string(),
        ));
    }

    let result = sqlx::query(
        "DELETE FROM project_collaborators WHERE project_id = $1 AND user_id = $2",
    )
    .bind(project_id)
    .bind(collaborator_user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Collaborator not found".to_string()));
    }

    Ok(())
}

/// List all active invite tokens for a project
pub async fn list_invite_tokens(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<Vec<InviteTokenResponse>> {
    // Check if user can edit project
    let can_edit = can_user_edit_project(pool, project_id, user_id).await?;
    if !can_edit {
        return Err(AppError::Forbidden);
    }

    let tokens = sqlx::query_as::<_, ProjectInviteToken>(
        "SELECT * FROM project_invite_tokens WHERE project_id = $1 AND is_active = TRUE ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?;

    Ok(tokens
        .into_iter()
        .map(|t| InviteTokenResponse {
            id: t.id,
            token: t.token.clone(),
            role: t.role,
            invite_url: format!("/editor?join={}", t.token),
            max_uses: t.max_uses,
            uses_count: t.uses_count,
            expires_at: t.expires_at,
            created_at: t.created_at,
            is_active: t.is_active,
        })
        .collect())
}

/// Deactivate an invite token
pub async fn deactivate_invite_token(
    pool: &PgPool,
    token_id: Uuid,
    user_id: Uuid,
) -> AppResult<()> {
    // Get token and verify user owns the project
    let token = sqlx::query_as::<_, ProjectInviteToken>(
        "SELECT * FROM project_invite_tokens WHERE id = $1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Invite token not found".to_string()))?;

    let can_edit = can_user_edit_project(pool, token.project_id, user_id).await?;
    if !can_edit {
        return Err(AppError::Forbidden);
    }

    sqlx::query("UPDATE project_invite_tokens SET is_active = FALSE WHERE id = $1")
        .bind(token_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Check if user can access a project (owner or collaborator)
pub async fn can_user_access_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<bool> {
    let can_access = sqlx::query_scalar::<_, bool>(
        "SELECT can_user_access_project($1, $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(can_access)
}

/// Check if user can edit a project (owner or editor collaborator)
pub async fn can_user_edit_project(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<bool> {
    let can_edit = sqlx::query_scalar::<_, bool>(
        "SELECT can_user_edit_project($1, $2)",
    )
    .bind(project_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(can_edit)
}
