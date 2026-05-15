//! Contact request handlers.
//!
//! Anonymous editor visitors POST the "Get access" form to a public endpoint;
//! admins read / mark them via the admin-only endpoints. No auth middleware
//! is applied to the create endpoint — that's intentional, the whole point
//! is to capture leads from users who aren't logged in.

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::models::{ContactRequest, CreateContactRequest, UpdateContactRequest};

/// POST /api/contact-requests — anonymous, no auth.
pub async fn create_contact_request(
    State(pool): State<DbPool>,
    Json(req): Json<CreateContactRequest>,
) -> AppResult<Json<ContactRequest>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let trimmed_phone = req
        .phone
        .as_ref()
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty());

    let row: ContactRequest = sqlx::query_as(
        r#"
        INSERT INTO contact_requests (name, email, phone, message)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, email, phone, message, contacted, contacted_at, created_at
        "#,
    )
    .bind(req.name.trim())
    .bind(req.email.trim().to_lowercase())
    .bind(trimmed_phone)
    .bind(req.message.trim())
    .fetch_one(&pool)
    .await?;

    Ok(Json(row))
}

/// GET /api/admin/contact-requests — admin only.
pub async fn list_contact_requests(
    State(pool): State<DbPool>,
) -> AppResult<Json<Vec<ContactRequest>>> {
    let rows: Vec<ContactRequest> = sqlx::query_as(
        r#"
        SELECT id, name, email, phone, message, contacted, contacted_at, created_at
        FROM contact_requests
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

/// PATCH /api/admin/contact-requests/:id — admin only. Flips the `contacted`
/// flag and stamps `contacted_at` accordingly, so an admin can keep track of
/// who's still waiting on a follow-up.
pub async fn update_contact_request(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateContactRequest>,
) -> AppResult<Json<ContactRequest>> {
    let row: Option<ContactRequest> = sqlx::query_as(
        r#"
        UPDATE contact_requests
        SET contacted = $2,
            contacted_at = CASE WHEN $2 THEN NOW() ELSE NULL END
        WHERE id = $1
        RETURNING id, name, email, phone, message, contacted, contacted_at, created_at
        "#,
    )
    .bind(id)
    .bind(req.contacted)
    .fetch_optional(&pool)
    .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("contact request".into()))
}
