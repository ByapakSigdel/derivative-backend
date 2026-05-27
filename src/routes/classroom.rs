//! Classroom / assignment / submission routes.
//!
//! These are split into three groups nested under distinct prefixes
//! (`/api/classrooms`, `/api/assignments`, `/api/submissions`) so each can be
//! mounted independently. All require authentication; the handlers enforce the
//! per-role (admin / org admin / teacher / student) access rules.

use axum::{
    routing::{delete, get, patch},
    Router,
};

use crate::db::DbPool;
use crate::handlers::classroom as h;

/// Routes mounted at `/api/classrooms`.
pub fn classroom_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(h::list_classrooms).post(h::create_classroom))
        .route(
            "/:id",
            get(h::get_classroom)
                .patch(h::update_classroom)
                .delete(h::delete_classroom),
        )
        .route(
            "/:id/students",
            get(h::list_students).post(h::enroll_student),
        )
        .route("/:id/students/:student_id", delete(h::unenroll_student))
        .route(
            "/:id/assignments",
            get(h::list_assignments).post(h::create_assignment),
        )
}

/// Routes mounted at `/api/assignments`.
pub fn assignment_routes() -> Router<DbPool> {
    Router::new()
        .route(
            "/:id",
            get(h::get_assignment)
                .patch(h::update_assignment)
                .delete(h::delete_assignment),
        )
        .route("/:id/submissions", get(h::list_submissions))
        .route(
            "/:id/submission",
            get(h::get_my_submission).put(h::upsert_my_submission),
        )
}

/// Routes mounted at `/api/submissions`.
pub fn submission_routes() -> Router<DbPool> {
    Router::new().route("/:id", patch(h::review_submission))
}
