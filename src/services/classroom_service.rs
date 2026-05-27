//! Classroom, assignment, and submission data operations.
//!
//! Authorization (who may call what) lives in the handler layer, which has the
//! caller's `AuthUser`. This module is purely the data layer; it returns rows
//! and enforces only data-integrity rules (uniqueness, existence, the
//! submitted-before-visible filter).

use sqlx::PgPool;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::models::{
    Assignment, Classroom, ClassroomDetail, ClassroomMember, CreateAssignmentRequest,
    CreateClassroomRequest, Submission, SubmissionDetail, UpdateAssignmentRequest,
    UpdateClassroomRequest,
};

// Column list shared by the plain Classroom selects.
const CLASSROOM_COLS: &str =
    "id, organization_id, teacher_id, name, description, created_at, updated_at";

const ASSIGNMENT_COLS: &str =
    "id, classroom_id, title, description, starter_project_id, due_at, created_at, updated_at";

const SUBMISSION_COLS: &str = "id, assignment_id, student_id, project_id, status, student_note, \
     grade, feedback, submitted_at, reviewed_at, created_at, updated_at";

// ---------------------------------------------------------------------------
// Classrooms
// ---------------------------------------------------------------------------

pub async fn create_classroom(
    pool: &PgPool,
    organization_id: Uuid,
    teacher_id: Uuid,
    request: &CreateClassroomRequest,
) -> AppResult<Classroom> {
    let classroom: Classroom = sqlx::query_as(
        r#"
        INSERT INTO classrooms (organization_id, teacher_id, name, description)
        VALUES ($1, $2, $3, $4)
        RETURNING id, organization_id, teacher_id, name, description, created_at, updated_at
        "#,
    )
    .bind(organization_id)
    .bind(teacher_id)
    .bind(&request.name)
    .bind(request.description.as_deref())
    .fetch_one(pool)
    .await?;

    Ok(classroom)
}

pub async fn get_classroom(pool: &PgPool, id: Uuid) -> AppResult<Classroom> {
    let sql = format!("SELECT {CLASSROOM_COLS} FROM classrooms WHERE id = $1");
    sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Classroom".to_string()))
}

pub async fn update_classroom(
    pool: &PgPool,
    id: Uuid,
    request: &UpdateClassroomRequest,
) -> AppResult<Classroom> {
    let classroom: Classroom = sqlx::query_as(
        r#"
        UPDATE classrooms
        SET name = COALESCE($1, name),
            description = COALESCE($2, description),
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, organization_id, teacher_id, name, description, created_at, updated_at
        "#,
    )
    .bind(request.name.as_deref())
    .bind(request.description.as_deref())
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Classroom".to_string()))?;

    Ok(classroom)
}

pub async fn delete_classroom(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM classrooms WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Classroom".to_string()));
    }
    Ok(())
}

/// Shared SELECT producing ClassroomDetail rows. `filter_sql` is a trusted,
/// code-defined WHERE fragment (never user input) that references `$1`.
async fn list_classroom_details(
    pool: &PgPool,
    filter_sql: &str,
    bind: Uuid,
) -> AppResult<Vec<ClassroomDetail>> {
    let sql = format!(
        r#"
        SELECT c.id, c.organization_id, c.teacher_id, t.full_name AS teacher_name,
               c.name, c.description,
               (SELECT COUNT(*) FROM classroom_students cs WHERE cs.classroom_id = c.id) AS student_count,
               (SELECT COUNT(*) FROM assignments a WHERE a.classroom_id = c.id) AS assignment_count,
               c.created_at, c.updated_at
        FROM classrooms c
        JOIN user_profiles t ON t.id = c.teacher_id
        {filter_sql}
        ORDER BY c.created_at DESC
        "#
    );

    let rows: Vec<ClassroomDetail> = sqlx::query_as(&sql).bind(bind).fetch_all(pool).await?;
    Ok(rows)
}

pub async fn list_classrooms_for_teacher(
    pool: &PgPool,
    teacher_id: Uuid,
) -> AppResult<Vec<ClassroomDetail>> {
    list_classroom_details(pool, "WHERE c.teacher_id = $1", teacher_id).await
}

pub async fn list_classrooms_for_org(
    pool: &PgPool,
    organization_id: Uuid,
) -> AppResult<Vec<ClassroomDetail>> {
    list_classroom_details(pool, "WHERE c.organization_id = $1", organization_id).await
}

pub async fn list_classrooms_for_student(
    pool: &PgPool,
    student_id: Uuid,
) -> AppResult<Vec<ClassroomDetail>> {
    list_classroom_details(
        pool,
        "WHERE c.id IN (SELECT classroom_id FROM classroom_students WHERE student_id = $1)",
        student_id,
    )
    .await
}

/// Every classroom on the platform — for platform-admin oversight.
pub async fn list_all_classrooms(pool: &PgPool) -> AppResult<Vec<ClassroomDetail>> {
    let rows: Vec<ClassroomDetail> = sqlx::query_as(
        r#"
        SELECT c.id, c.organization_id, c.teacher_id, t.full_name AS teacher_name,
               c.name, c.description,
               (SELECT COUNT(*) FROM classroom_students cs WHERE cs.classroom_id = c.id) AS student_count,
               (SELECT COUNT(*) FROM assignments a WHERE a.classroom_id = c.id) AS assignment_count,
               c.created_at, c.updated_at
        FROM classrooms c
        JOIN user_profiles t ON t.id = c.teacher_id
        ORDER BY c.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Enrollment
// ---------------------------------------------------------------------------

pub async fn is_enrolled(pool: &PgPool, classroom_id: Uuid, student_id: Uuid) -> AppResult<bool> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM classroom_students WHERE classroom_id = $1 AND student_id = $2",
    )
    .bind(classroom_id)
    .bind(student_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.is_some())
}

pub async fn enroll_student(
    pool: &PgPool,
    classroom_id: Uuid,
    student_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO classroom_students (classroom_id, student_id)
        VALUES ($1, $2)
        ON CONFLICT (classroom_id, student_id) DO NOTHING
        "#,
    )
    .bind(classroom_id)
    .bind(student_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn unenroll_student(
    pool: &PgPool,
    classroom_id: Uuid,
    student_id: Uuid,
) -> AppResult<()> {
    let result =
        sqlx::query("DELETE FROM classroom_students WHERE classroom_id = $1 AND student_id = $2")
            .bind(classroom_id)
            .bind(student_id)
            .execute(pool)
            .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Enrollment".to_string()));
    }
    Ok(())
}

pub async fn list_classroom_students(
    pool: &PgPool,
    classroom_id: Uuid,
) -> AppResult<Vec<ClassroomMember>> {
    let rows: Vec<ClassroomMember> = sqlx::query_as(
        r#"
        SELECT u.id, u.email, u.full_name, u.is_active, cs.created_at AS enrolled_at
        FROM classroom_students cs
        JOIN user_profiles u ON u.id = cs.student_id
        WHERE cs.classroom_id = $1
        ORDER BY u.full_name ASC
        "#,
    )
    .bind(classroom_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ---------------------------------------------------------------------------
// Assignments
// ---------------------------------------------------------------------------

pub async fn create_assignment(
    pool: &PgPool,
    classroom_id: Uuid,
    request: &CreateAssignmentRequest,
) -> AppResult<Assignment> {
    let assignment: Assignment = sqlx::query_as(
        r#"
        INSERT INTO assignments (classroom_id, title, description, starter_project_id, due_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, classroom_id, title, description, starter_project_id, due_at,
                  created_at, updated_at
        "#,
    )
    .bind(classroom_id)
    .bind(&request.title)
    .bind(request.description.as_deref())
    .bind(request.starter_project_id)
    .bind(request.due_at)
    .fetch_one(pool)
    .await?;
    Ok(assignment)
}

pub async fn get_assignment(pool: &PgPool, id: Uuid) -> AppResult<Assignment> {
    let sql = format!("SELECT {ASSIGNMENT_COLS} FROM assignments WHERE id = $1");
    sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Assignment".to_string()))
}

pub async fn list_assignments(pool: &PgPool, classroom_id: Uuid) -> AppResult<Vec<Assignment>> {
    let sql = format!(
        "SELECT {ASSIGNMENT_COLS} FROM assignments WHERE classroom_id = $1 ORDER BY created_at DESC"
    );
    let rows: Vec<Assignment> = sqlx::query_as(&sql)
        .bind(classroom_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn update_assignment(
    pool: &PgPool,
    id: Uuid,
    request: &UpdateAssignmentRequest,
) -> AppResult<Assignment> {
    let assignment: Assignment = sqlx::query_as(
        r#"
        UPDATE assignments
        SET title = COALESCE($1, title),
            description = COALESCE($2, description),
            starter_project_id = COALESCE($3, starter_project_id),
            due_at = COALESCE($4, due_at),
            updated_at = NOW()
        WHERE id = $5
        RETURNING id, classroom_id, title, description, starter_project_id, due_at,
                  created_at, updated_at
        "#,
    )
    .bind(request.title.as_deref())
    .bind(request.description.as_deref())
    .bind(request.starter_project_id)
    .bind(request.due_at)
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Assignment".to_string()))?;
    Ok(assignment)
}

pub async fn delete_assignment(pool: &PgPool, id: Uuid) -> AppResult<()> {
    let result = sqlx::query("DELETE FROM assignments WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Assignment".to_string()));
    }
    Ok(())
}

/// True if the given project exists and is owned by `user_id`. Used to stop a
/// student attaching someone else's project to a submission.
pub async fn project_belongs_to(
    pool: &PgPool,
    project_id: Uuid,
    user_id: Uuid,
) -> AppResult<bool> {
    let row: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM user_projects WHERE id = $1 AND user_id = $2")
            .bind(project_id)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    Ok(row.is_some())
}

/// True if `reviewer` may read `project_id` because it's attached to a
/// submission they're allowed to review — i.e. the submission is past
/// in_progress (the student marked it done) and lives in a classroom the
/// reviewer manages (owning teacher, org admin of that org, or platform admin).
///
/// This is the authorization behind "a teacher can open a student's private
/// project to grade it" (see handlers::projects::get_project).
pub async fn project_is_reviewable_by(
    pool: &PgPool,
    project_id: Uuid,
    reviewer_id: Uuid,
    reviewer_is_admin: bool,
    reviewer_is_org_admin: bool,
    reviewer_org: Option<Uuid>,
) -> AppResult<bool> {
    let row: (bool,) = sqlx::query_as(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM submissions s
            JOIN assignments a ON a.id = s.assignment_id
            JOIN classrooms c ON c.id = a.classroom_id
            WHERE s.project_id = $1
              AND s.status <> 'in_progress'
              AND (
                $2 = TRUE
                OR c.teacher_id = $3
                OR ($4 = TRUE AND $5::uuid IS NOT NULL AND c.organization_id = $5)
              )
        )
        "#,
    )
    .bind(project_id)
    .bind(reviewer_is_admin)
    .bind(reviewer_id)
    .bind(reviewer_is_org_admin)
    .bind(reviewer_org)
    .fetch_one(pool)
    .await?;
    Ok(row.0)
}

/// Fetch the classroom that owns an assignment (for authorization checks).
pub async fn get_classroom_for_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
) -> AppResult<Classroom> {
    sqlx::query_as(
        r#"
        SELECT c.id, c.organization_id, c.teacher_id, c.name, c.description,
               c.created_at, c.updated_at
        FROM classrooms c
        JOIN assignments a ON a.classroom_id = c.id
        WHERE a.id = $1
        "#,
    )
    .bind(assignment_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Assignment".to_string()))
}

// ---------------------------------------------------------------------------
// Submissions
// ---------------------------------------------------------------------------

/// Create or update a student's submission for an assignment. When `submit` is
/// true the status moves to 'submitted' and `submitted_at` is stamped; the
/// student's project/note are preserved if not supplied (COALESCE), so a
/// partial save never wipes prior content.
pub async fn upsert_submission(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
    project_id: Option<Uuid>,
    student_note: Option<&str>,
    submit: bool,
) -> AppResult<Submission> {
    let initial_status = if submit { "submitted" } else { "in_progress" };

    let submission: Submission = sqlx::query_as(
        r#"
        INSERT INTO submissions
            (assignment_id, student_id, project_id, student_note, status, submitted_at)
        VALUES ($1, $2, $3, $4, $5, CASE WHEN $6 THEN NOW() ELSE NULL END)
        ON CONFLICT (assignment_id, student_id) DO UPDATE SET
            project_id = COALESCE(EXCLUDED.project_id, submissions.project_id),
            student_note = COALESCE(EXCLUDED.student_note, submissions.student_note),
            status = CASE WHEN $6 THEN 'submitted' ELSE submissions.status END,
            submitted_at = CASE WHEN $6 THEN NOW() ELSE submissions.submitted_at END,
            updated_at = NOW()
        RETURNING id, assignment_id, student_id, project_id, status, student_note,
                  grade, feedback, submitted_at, reviewed_at, created_at, updated_at
        "#,
    )
    .bind(assignment_id)
    .bind(student_id)
    .bind(project_id)
    .bind(student_note)
    .bind(initial_status)
    .bind(submit)
    .fetch_one(pool)
    .await?;

    Ok(submission)
}

pub async fn get_submission_for_student(
    pool: &PgPool,
    assignment_id: Uuid,
    student_id: Uuid,
) -> AppResult<Option<Submission>> {
    let sql = format!(
        "SELECT {SUBMISSION_COLS} FROM submissions WHERE assignment_id = $1 AND student_id = $2"
    );
    let row: Option<Submission> = sqlx::query_as(&sql)
        .bind(assignment_id)
        .bind(student_id)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

pub async fn get_submission(pool: &PgPool, id: Uuid) -> AppResult<Submission> {
    let sql = format!("SELECT {SUBMISSION_COLS} FROM submissions WHERE id = $1");
    sqlx::query_as(&sql)
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::NotFound("Submission".to_string()))
}

/// List submissions for an assignment. When `visible_only` is true (the
/// teacher's view) submissions still in progress are hidden — the teacher only
/// sees work the student has marked done.
pub async fn list_submissions_for_assignment(
    pool: &PgPool,
    assignment_id: Uuid,
    visible_only: bool,
) -> AppResult<Vec<SubmissionDetail>> {
    let visibility = if visible_only {
        "AND s.status <> 'in_progress'"
    } else {
        ""
    };
    let sql = format!(
        r#"
        SELECT s.id, s.assignment_id, s.student_id,
               u.full_name AS student_name, u.email AS student_email,
               s.project_id, s.status, s.student_note, s.grade, s.feedback,
               s.submitted_at, s.reviewed_at, s.created_at, s.updated_at
        FROM submissions s
        JOIN user_profiles u ON u.id = s.student_id
        WHERE s.assignment_id = $1 {visibility}
        ORDER BY s.submitted_at DESC NULLS LAST, u.full_name ASC
        "#
    );
    let rows: Vec<SubmissionDetail> = sqlx::query_as(&sql)
        .bind(assignment_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

/// Teacher review: set optional grade/feedback and move to 'reviewed'.
pub async fn review_submission(
    pool: &PgPool,
    id: Uuid,
    grade: Option<&str>,
    feedback: Option<&str>,
) -> AppResult<Submission> {
    let submission: Submission = sqlx::query_as(
        r#"
        UPDATE submissions
        SET grade = COALESCE($1, grade),
            feedback = COALESCE($2, feedback),
            status = 'reviewed',
            reviewed_at = NOW(),
            updated_at = NOW()
        WHERE id = $3
        RETURNING id, assignment_id, student_id, project_id, status, student_note,
                  grade, feedback, submitted_at, reviewed_at, created_at, updated_at
        "#,
    )
    .bind(grade)
    .bind(feedback)
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound("Submission".to_string()))?;
    Ok(submission)
}
