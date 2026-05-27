//! Classroom, assignment, and submission models for the teaching workflow.
//!
//! Ownership chain: organization → classroom (owned by a teacher) → assignment
//! → submission (one per enrolled student). Org admins oversee their org's
//! classrooms; teachers run their own; students work and submit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

// ---------------------------------------------------------------------------
// Classrooms
// ---------------------------------------------------------------------------

/// A classroom row.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Classroom {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub teacher_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Classroom enriched with the teacher's name and roster/assignment counts,
/// for list views.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ClassroomDetail {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub teacher_id: Uuid,
    pub teacher_name: String,
    pub name: String,
    pub description: Option<String>,
    pub student_count: i64,
    pub assignment_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateClassroomRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateClassroomRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 2000, message = "Description must be at most 2000 characters"))]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Enrollment
// ---------------------------------------------------------------------------

/// A student enrolled in a classroom, joined with their profile for display.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ClassroomMember {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub is_active: bool,
    pub enrolled_at: DateTime<Utc>,
}

/// Enroll a student by their existing user id OR by email. Exactly one is
/// required; the handler resolves email → id and verifies same-org membership.
#[derive(Debug, Deserialize)]
pub struct EnrollStudentRequest {
    #[serde(default)]
    pub student_id: Option<Uuid>,
    #[serde(default)]
    pub email: Option<String>,
}

// ---------------------------------------------------------------------------
// Assignments
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Assignment {
    pub id: Uuid,
    pub classroom_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub starter_project_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAssignmentRequest {
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub title: String,
    #[validate(length(max = 5000, message = "Description must be at most 5000 characters"))]
    pub description: Option<String>,
    pub starter_project_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAssignmentRequest {
    #[validate(length(min = 1, max = 255, message = "Title must be between 1 and 255 characters"))]
    pub title: Option<String>,
    #[validate(length(max = 5000, message = "Description must be at most 5000 characters"))]
    pub description: Option<String>,
    pub starter_project_id: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
}

// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Submission {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub project_id: Option<Uuid>,
    pub status: String,
    pub student_note: Option<String>,
    pub grade: Option<String>,
    pub feedback: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Submission joined with the student's display fields — the shape the teacher
/// sees in the review list.
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct SubmissionDetail {
    pub id: Uuid,
    pub assignment_id: Uuid,
    pub student_id: Uuid,
    pub student_name: String,
    pub student_email: String,
    pub project_id: Option<Uuid>,
    pub status: String,
    pub student_note: Option<String>,
    pub grade: Option<String>,
    pub feedback: Option<String>,
    pub submitted_at: Option<DateTime<Utc>>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A student creating or updating their own submission. `submit = true` flips
/// the status to 'submitted' (marks it done — now visible to the teacher).
#[derive(Debug, Deserialize)]
pub struct UpsertSubmissionRequest {
    #[serde(default)]
    pub project_id: Option<Uuid>,
    #[serde(default)]
    pub student_note: Option<String>,
    #[serde(default)]
    pub submit: bool,
}

/// A teacher reviewing a submission: optional grade + feedback. Applying it
/// moves the submission to 'reviewed'.
#[derive(Debug, Deserialize, Validate)]
pub struct ReviewSubmissionRequest {
    #[validate(length(max = 50, message = "Grade must be at most 50 characters"))]
    pub grade: Option<String>,
    #[validate(length(max = 5000, message = "Feedback must be at most 5000 characters"))]
    pub feedback: Option<String>,
}
