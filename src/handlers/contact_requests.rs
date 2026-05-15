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
use crate::models::{
    ContactRequest, CreateContactRequest, CreateContactResponse, UpdateContactRequest,
};

/// POST /api/contact-requests — anonymous, no auth.
///
/// Dedupe: identical (lower(email), phone, coalesce(message,'')) is
/// considered the same request thanks to the unique index added in
/// migration 018. We do a two-step "insert if new, otherwise fetch" so the
/// response always carries a row + a `created` flag the caller can use to
/// decide whether to send a confirmation email.
pub async fn create_contact_request(
    State(pool): State<DbPool>,
    Json(req): Json<CreateContactRequest>,
) -> AppResult<Json<CreateContactResponse>> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let name = req.name.trim().to_string();
    let email = req.email.trim().to_lowercase();
    let phone = req.phone.trim().to_string();
    let user_type = req.user_type.trim().to_lowercase();
    // Empty/whitespace message → NULL so the dedupe index treats "no
    // message" consistently regardless of how the client sent it.
    let message = req
        .message
        .as_ref()
        .map(|m| m.trim().to_string())
        .filter(|m| !m.is_empty());

    // Try the insert. ON CONFLICT DO NOTHING means a duplicate payload
    // returns zero rows — we then fetch the existing one separately.
    let inserted: Option<ContactRequest> = sqlx::query_as(
        r#"
        INSERT INTO contact_requests (name, email, phone, message, user_type)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (lower(email), phone, COALESCE(message, ''))
        DO NOTHING
        RETURNING id, name, email, phone, message, user_type,
                  contacted, contacted_at, created_at
        "#,
    )
    .bind(&name)
    .bind(&email)
    .bind(&phone)
    .bind(message.as_deref())
    .bind(&user_type)
    .fetch_optional(&pool)
    .await?;

    if let Some(row) = inserted {
        return Ok(Json(CreateContactResponse {
            created: true,
            request: row,
        }));
    }

    // No row inserted ⇒ dupe. Find the existing row so the caller still
    // gets a useful response (and so the frontend can short-circuit the
    // "thanks" state instead of erroring).
    let existing: ContactRequest = sqlx::query_as(
        r#"
        SELECT id, name, email, phone, message, user_type,
               contacted, contacted_at, created_at
        FROM contact_requests
        WHERE lower(email) = $1 AND phone = $2 AND COALESCE(message, '') = $3
        LIMIT 1
        "#,
    )
    .bind(&email)
    .bind(&phone)
    .bind(message.as_deref().unwrap_or(""))
    .fetch_one(&pool)
    .await?;

    Ok(Json(CreateContactResponse {
        created: false,
        request: existing,
    }))
}

/// GET /api/admin/contact-requests — admin only.
pub async fn list_contact_requests(
    State(pool): State<DbPool>,
) -> AppResult<Json<Vec<ContactRequest>>> {
    let rows: Vec<ContactRequest> = sqlx::query_as(
        r#"
        SELECT id, name, email, phone, message, user_type,
               contacted, contacted_at, created_at
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
        RETURNING id, name, email, phone, message, user_type,
                  contacted, contacted_at, created_at
        "#,
    )
    .bind(id)
    .bind(req.contacted)
    .fetch_optional(&pool)
    .await?;

    row.map(Json)
        .ok_or_else(|| AppError::NotFound("contact request".into()))
}
