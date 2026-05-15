//! Contact request model.
//!
//! "Get access" form submissions from anonymous trial users.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

/// Contact request row.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ContactRequest {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub message: Option<String>,
    pub user_type: String,
    pub contacted: bool,
    pub contacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Public POST body. Anonymous endpoint, so we validate aggressively here —
/// it's the only line of defence against junk.
///
/// Fields are intentionally NOT all required: phone is mandatory (the
/// support flow uses callbacks), message is optional (some people just
/// want a callback and have nothing specific to write).
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 1, max = 120, message = "Name must be 1..120 chars"))]
    pub name: String,
    #[validate(email(message = "Invalid email"))]
    #[validate(length(max = 255, message = "Email too long"))]
    pub email: String,
    #[validate(length(min = 3, max = 60, message = "Phone must be 3..60 chars"))]
    pub phone: String,
    #[validate(length(max = 4000, message = "Message too long"))]
    pub message: Option<String>,
    /// "individual" | "organization". Free-form string in the DB so we can
    /// extend the option list without another migration.
    #[validate(length(min = 1, max = 40, message = "Invalid user type"))]
    pub user_type: String,
}

/// Wrapper response so the handler can tell the caller whether the insert
/// was a real one (`created = true`, send confirmation email) or a dedupe
/// no-op (`created = false`, the same payload was already on file).
#[derive(Debug, Serialize)]
pub struct CreateContactResponse {
    pub created: bool,
    pub request: ContactRequest,
}

/// Admin PATCH body — currently just toggles the "contacted" flag.
#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub contacted: bool,
}
