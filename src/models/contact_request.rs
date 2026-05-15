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
    pub phone: Option<String>,
    pub message: String,
    pub contacted: bool,
    pub contacted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Public POST body. Anonymous endpoint, so we validate aggressively here —
/// it's the only line of defence against junk.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateContactRequest {
    #[validate(length(min = 1, max = 120, message = "Name must be 1..120 chars"))]
    pub name: String,
    #[validate(email(message = "Invalid email"))]
    #[validate(length(max = 255, message = "Email too long"))]
    pub email: String,
    #[validate(length(max = 60, message = "Phone too long"))]
    pub phone: Option<String>,
    #[validate(length(min = 1, max = 4000, message = "Message must be 1..4000 chars"))]
    pub message: String,
}

/// Admin PATCH body — currently just toggles the "contacted" flag.
#[derive(Debug, Deserialize)]
pub struct UpdateContactRequest {
    pub contacted: bool,
}
