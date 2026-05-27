//! Classroom, assignment, and submission handlers.
//!
//! Authorization model (per request, using the caller's token):
//!   * manage  = platform admin, OR org admin of the classroom's org, OR the
//!               teacher who owns the classroom.
//!   * view    = anyone who can manage, OR a student enrolled in the classroom.
//!
//! The headline rule — a teacher only sees a student's work once it's marked
//! done — is enforced by listing submissions with `visible_only = true`, which
//! filters out `in_progress` rows.

use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;
use validator::Validate;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::{AuthUser, CurrentUser};
use crate::models::{
    Assignment, Classroom, ClassroomDetail, ClassroomMember, CreateAssignmentRequest,
    CreateClassroomRequest, EnrollStudentRequest, ReviewSubmissionRequest, Submission,
    SubmissionDetail, UpdateAssignmentRequest, UpdateClassroomRequest, UpsertSubmissionRequest,
    UserType,
};
use crate::services::{classroom_service, user_service};

#[derive(serde::Serialize)]
pub struct MessageResponse {
    message: String,
}

fn ok_message(message: &str) -> Json<MessageResponse> {
    Json(MessageResponse {
        message: message.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Authorization helpers
// ---------------------------------------------------------------------------

/// Can this user create/update/delete within the classroom?
fn can_manage(user: &AuthUser, classroom: &Classroom) -> bool {
    user.is_admin()
        || user.can_administer_org(classroom.organization_id)
        || (user.is_teacher() && classroom.teacher_id == user.id)
}

fn ensure_manage(user: &AuthUser, classroom: &Classroom) -> AppResult<()> {
    if can_manage(user, classroom) {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

/// Can this user view the classroom (managers + enrolled students)?
async fn ensure_view(pool: &DbPool, user: &AuthUser, classroom: &Classroom) -> AppResult<()> {
    if can_manage(user, classroom) {
        return Ok(());
    }
    if user.is_student() && classroom_service::is_enrolled(pool, classroom.id, user.id).await? {
        return Ok(());
    }
    Err(AppError::Forbidden)
}

// ---------------------------------------------------------------------------
// Classrooms
// ---------------------------------------------------------------------------

/// Create a classroom. Teachers create classrooms they own, within their org.
pub async fn create_classroom(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Json(request): Json<CreateClassroomRequest>,
) -> AppResult<Json<Classroom>> {
    if !user.auth().is_teacher() {
        return Err(AppError::Forbidden);
    }
    let org_id = user.organization_id().ok_or(AppError::Forbidden)?;
    request.validate()?;
    let classroom =
        classroom_service::create_classroom(&pool, org_id, user.id(), &request).await?;
    Ok(Json(classroom))
}

/// List classrooms visible to the caller, depending on their role.
pub async fn list_classrooms(
    State(pool): State<DbPool>,
    user: CurrentUser,
) -> AppResult<Json<Vec<ClassroomDetail>>> {
    let auth = user.auth();
    let classrooms = if auth.is_admin() {
        classroom_service::list_all_classrooms(&pool).await?
    } else if auth.is_org_admin() {
        let org_id = auth.organization_id.ok_or(AppError::Forbidden)?;
        classroom_service::list_classrooms_for_org(&pool, org_id).await?
    } else if auth.is_teacher() {
        classroom_service::list_classrooms_for_teacher(&pool, auth.id).await?
    } else if auth.is_student() {
        classroom_service::list_classrooms_for_student(&pool, auth.id).await?
    } else {
        Vec::new()
    };
    Ok(Json(classrooms))
}

pub async fn get_classroom(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Classroom>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_view(&pool, user.auth(), &classroom).await?;
    Ok(Json(classroom))
}

pub async fn update_classroom(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateClassroomRequest>,
) -> AppResult<Json<Classroom>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_manage(user.auth(), &classroom)?;
    request.validate()?;
    let updated = classroom_service::update_classroom(&pool, id, &request).await?;
    Ok(Json(updated))
}

pub async fn delete_classroom(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_manage(user.auth(), &classroom)?;
    classroom_service::delete_classroom(&pool, id).await?;
    Ok(ok_message("Classroom deleted successfully"))
}

// ---------------------------------------------------------------------------
// Enrollment
// ---------------------------------------------------------------------------

pub async fn list_students(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<ClassroomMember>>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_view(&pool, user.auth(), &classroom).await?;
    let students = classroom_service::list_classroom_students(&pool, id).await?;
    Ok(Json(students))
}

/// Enroll a student (by id or email). The student must be a `student` in the
/// same organization as the classroom.
pub async fn enroll_student(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(id): Path<Uuid>,
    Json(request): Json<EnrollStudentRequest>,
) -> AppResult<Json<MessageResponse>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_manage(user.auth(), &classroom)?;

    // Resolve the target student by id or email.
    let student = if let Some(sid) = request.student_id {
        user_service::get_user_by_id(&pool, sid).await?
    } else if let Some(ref email) = request.email {
        user_service::get_user_by_email(&pool, email)
            .await?
            .ok_or(AppError::NotFound("User".to_string()))?
    } else {
        return Err(AppError::BadRequest(
            "Provide a student_id or email to enroll".to_string(),
        ));
    };

    if student.user_type != UserType::Student
        || student.organization_id != Some(classroom.organization_id)
    {
        return Err(AppError::BadRequest(
            "Only students in this organization can be enrolled".to_string(),
        ));
    }

    classroom_service::enroll_student(&pool, id, student.id).await?;
    Ok(ok_message("Student enrolled successfully"))
}

pub async fn unenroll_student(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path((id, student_id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<MessageResponse>> {
    let classroom = classroom_service::get_classroom(&pool, id).await?;
    ensure_manage(user.auth(), &classroom)?;
    classroom_service::unenroll_student(&pool, id, student_id).await?;
    Ok(ok_message("Student removed from classroom"))
}

// ---------------------------------------------------------------------------
// Assignments
// ---------------------------------------------------------------------------

pub async fn create_assignment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(classroom_id): Path<Uuid>,
    Json(request): Json<CreateAssignmentRequest>,
) -> AppResult<Json<Assignment>> {
    let classroom = classroom_service::get_classroom(&pool, classroom_id).await?;
    ensure_manage(user.auth(), &classroom)?;
    request.validate()?;
    let assignment = classroom_service::create_assignment(&pool, classroom_id, &request).await?;
    Ok(Json(assignment))
}

pub async fn list_assignments(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(classroom_id): Path<Uuid>,
) -> AppResult<Json<Vec<Assignment>>> {
    let classroom = classroom_service::get_classroom(&pool, classroom_id).await?;
    ensure_view(&pool, user.auth(), &classroom).await?;
    let assignments = classroom_service::list_assignments(&pool, classroom_id).await?;
    Ok(Json(assignments))
}

pub async fn get_assignment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
) -> AppResult<Json<Assignment>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    ensure_view(&pool, user.auth(), &classroom).await?;
    let assignment = classroom_service::get_assignment(&pool, assignment_id).await?;
    Ok(Json(assignment))
}

pub async fn update_assignment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
    Json(request): Json<UpdateAssignmentRequest>,
) -> AppResult<Json<Assignment>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    ensure_manage(user.auth(), &classroom)?;
    request.validate()?;
    let assignment = classroom_service::update_assignment(&pool, assignment_id, &request).await?;
    Ok(Json(assignment))
}

pub async fn delete_assignment(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    ensure_manage(user.auth(), &classroom)?;
    classroom_service::delete_assignment(&pool, assignment_id).await?;
    Ok(ok_message("Assignment deleted successfully"))
}

// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

/// Teacher view: list submissions that students have marked done. In-progress
/// work is deliberately hidden (`visible_only = true`).
pub async fn list_submissions(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
) -> AppResult<Json<Vec<SubmissionDetail>>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    ensure_manage(user.auth(), &classroom)?;
    let submissions =
        classroom_service::list_submissions_for_assignment(&pool, assignment_id, true).await?;
    Ok(Json(submissions))
}

/// Student view: fetch my own submission for an assignment (may be null).
pub async fn get_my_submission(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
) -> AppResult<Json<Option<Submission>>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    if !(user.auth().is_student()
        && classroom_service::is_enrolled(&pool, classroom.id, user.id()).await?)
    {
        return Err(AppError::Forbidden);
    }
    let submission =
        classroom_service::get_submission_for_student(&pool, assignment_id, user.id()).await?;
    Ok(Json(submission))
}

/// Student action: create/update my submission. `submit = true` marks it done,
/// making it visible to the teacher.
pub async fn upsert_my_submission(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(assignment_id): Path<Uuid>,
    Json(request): Json<UpsertSubmissionRequest>,
) -> AppResult<Json<Submission>> {
    let classroom = classroom_service::get_classroom_for_assignment(&pool, assignment_id).await?;
    if !(user.auth().is_student()
        && classroom_service::is_enrolled(&pool, classroom.id, user.id()).await?)
    {
        return Err(AppError::Forbidden);
    }

    // A student may only attach their own project.
    if let Some(project_id) = request.project_id {
        if !classroom_service::project_belongs_to(&pool, project_id, user.id()).await? {
            return Err(AppError::BadRequest(
                "Project not found or not owned by you".to_string(),
            ));
        }
    }

    let submission = classroom_service::upsert_submission(
        &pool,
        assignment_id,
        user.id(),
        request.project_id,
        request.student_note.as_deref(),
        request.submit,
    )
    .await?;
    Ok(Json(submission))
}

/// Teacher action: grade / give feedback on a submission, moving it to
/// 'reviewed'. Only submissions the student has marked done are reviewable.
pub async fn review_submission(
    State(pool): State<DbPool>,
    user: CurrentUser,
    Path(submission_id): Path<Uuid>,
    Json(request): Json<ReviewSubmissionRequest>,
) -> AppResult<Json<Submission>> {
    let submission = classroom_service::get_submission(&pool, submission_id).await?;
    let classroom =
        classroom_service::get_classroom_for_assignment(&pool, submission.assignment_id).await?;
    ensure_manage(user.auth(), &classroom)?;
    request.validate()?;

    // Can't review work the student hasn't submitted yet.
    if submission.status == "in_progress" {
        return Err(AppError::NotFound("Submission".to_string()));
    }

    let reviewed = classroom_service::review_submission(
        &pool,
        submission_id,
        request.grade.as_deref(),
        request.feedback.as_deref(),
    )
    .await?;
    Ok(Json(reviewed))
}
