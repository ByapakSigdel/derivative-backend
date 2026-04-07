//! Metrics and analytics service

use crate::errors::{AppError, AppResult};
use crate::models::{
    CategoryStats, DashboardMetrics, DifficultyStats, LogCompilationRequest, LogUploadRequest,
    MetricsTimeSeries, SystemMetrics, TimeSeriesPoint, TopProject, UserActivity,
};
use chrono::{Duration, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Get dashboard overview metrics for admin
pub async fn get_dashboard_metrics(pool: &PgPool) -> AppResult<DashboardMetrics> {
    // Use individual queries for robustness when some tables might not exist
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM user_profiles")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let total_projects: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM user_projects")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let public_projects: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM user_projects WHERE is_public = TRUE",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let private_projects: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM user_projects WHERE is_public = FALSE",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let total_organizations: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM organizations")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let total_views: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(view_count), 0)::BIGINT FROM user_projects",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let total_likes: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(like_count), 0)::BIGINT FROM user_projects",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let total_comments: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(comment_count), 0)::BIGINT FROM user_projects",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    // These might fail if tables don't exist yet - use default 0
    let total_compilations: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM compilation_logs")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let successful_compilations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM compilation_logs WHERE status = 'success'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let total_uploads: i64 = sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM upload_logs")
        .fetch_one(pool)
        .await
        .unwrap_or(0);

    let successful_uploads: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM upload_logs WHERE status = 'success'",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let featured_projects: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM user_projects WHERE is_featured = TRUE",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    Ok(DashboardMetrics {
        total_users,
        total_projects,
        public_projects,
        private_projects,
        total_organizations,
        total_views,
        total_likes,
        total_comments,
        total_compilations,
        successful_compilations,
        total_uploads,
        successful_uploads,
        featured_projects,
    })
}

/// Get metrics time series for charts (last 30 days)
pub async fn get_metrics_time_series(pool: &PgPool, days: i32) -> AppResult<MetricsTimeSeries> {
    let start_date = Utc::now().date_naive() - Duration::days(days as i64);

    // Update metrics for today if not exists
    sqlx::query("SELECT update_daily_metrics()")
        .execute(pool)
        .await
        .ok(); // Ignore errors

    let metrics = sqlx::query_as::<_, SystemMetrics>(
        "SELECT * FROM system_metrics WHERE metric_date >= $1 ORDER BY metric_date ASC",
    )
    .bind(start_date)
    .fetch_all(pool)
    .await?;

    Ok(MetricsTimeSeries {
        users: metrics
            .iter()
            .map(|m| TimeSeriesPoint {
                date: m.metric_date,
                value: m.new_users,
            })
            .collect(),
        projects: metrics
            .iter()
            .map(|m| TimeSeriesPoint {
                date: m.metric_date,
                value: m.new_projects,
            })
            .collect(),
        compilations: metrics
            .iter()
            .map(|m| TimeSeriesPoint {
                date: m.metric_date,
                value: m.total_compilations,
            })
            .collect(),
        uploads: metrics
            .iter()
            .map(|m| TimeSeriesPoint {
                date: m.metric_date,
                value: m.total_uploads,
            })
            .collect(),
    })
}

/// Get top projects by views
pub async fn get_top_projects_by_views(pool: &PgPool, limit: i32) -> AppResult<Vec<TopProject>> {
    let projects = sqlx::query_as::<_, TopProject>(
        r#"
        SELECT 
            p.id,
            p.title,
            u.email as author_email,
            p.view_count as value,
            p.created_at
        FROM user_projects p
        JOIN user_profiles u ON u.id = p.user_id
        WHERE p.is_public = TRUE
        ORDER BY p.view_count DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

/// Get top projects by likes
pub async fn get_top_projects_by_likes(pool: &PgPool, limit: i32) -> AppResult<Vec<TopProject>> {
    let projects = sqlx::query_as::<_, TopProject>(
        r#"
        SELECT 
            p.id,
            p.title,
            u.email as author_email,
            p.like_count as value,
            p.created_at
        FROM user_projects p
        JOIN user_profiles u ON u.id = p.user_id
        WHERE p.is_public = TRUE
        ORDER BY p.like_count DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(projects)
}

/// Get category distribution
pub async fn get_category_stats(pool: &PgPool) -> AppResult<Vec<CategoryStats>> {
    let stats = sqlx::query_as::<_, CategoryStats>(
        r#"
        SELECT 
            category::text as category,
            COUNT(*) as count
        FROM user_projects
        GROUP BY category
        ORDER BY count DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

/// Get difficulty distribution
pub async fn get_difficulty_stats(pool: &PgPool) -> AppResult<Vec<DifficultyStats>> {
    let stats = sqlx::query_as::<_, DifficultyStats>(
        r#"
        SELECT 
            difficulty::text as difficulty,
            COUNT(*) as count
        FROM user_projects
        GROUP BY difficulty
        ORDER BY count DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(stats)
}

/// Get most active users
pub async fn get_top_users(pool: &PgPool, limit: i32) -> AppResult<Vec<UserActivity>> {
    let users = sqlx::query_as::<_, UserActivity>(
        r#"
        SELECT 
            u.id as user_id,
            u.email as user_email,
            u.full_name as user_name,
            COUNT(p.id) as project_count,
            COALESCE(SUM(p.view_count), 0) as total_views,
            COALESCE(SUM(p.like_count), 0) as total_likes,
            (SELECT COUNT(*) FROM compilation_logs WHERE user_id = u.id) as total_compilations
        FROM user_profiles u
        LEFT JOIN user_projects p ON p.user_id = u.id
        GROUP BY u.id, u.email, u.full_name
        ORDER BY project_count DESC, total_views DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(users)
}

/// Log a compilation attempt
pub async fn log_compilation(
    pool: &PgPool,
    user_id: Uuid,
    req: LogCompilationRequest,
) -> AppResult<Uuid> {
    let log_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT log_compilation($1, $2, $3, $4, $5, $6, $7, $8)
        "#,
    )
    .bind(req.project_id)
    .bind(user_id)
    .bind(&req.status)
    .bind(&req.error_message)
    .bind(req.compilation_time_ms)
    .bind(req.code_size_bytes)
    .bind(req.node_count)
    .bind(req.edge_count)
    .fetch_one(pool)
    .await?;

    Ok(log_id)
}

/// Log an upload attempt
pub async fn log_upload(pool: &PgPool, user_id: Uuid, req: LogUploadRequest) -> AppResult<Uuid> {
    let log_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT log_upload($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(req.project_id)
    .bind(user_id)
    .bind(&req.board_type)
    .bind(&req.port)
    .bind(&req.status)
    .bind(&req.error_message)
    .bind(req.upload_time_ms)
    .fetch_one(pool)
    .await?;

    Ok(log_id)
}

/// Update daily metrics (can be called via cron or manually)
pub async fn update_daily_metrics(pool: &PgPool) -> AppResult<()> {
    sqlx::query("SELECT update_daily_metrics()")
        .execute(pool)
        .await?;

    Ok(())
}
