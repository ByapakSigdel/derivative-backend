//! Analytics and metrics models

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Compilation log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CompilationLog {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub status: String, // 'success', 'error', 'warning'
    pub error_message: Option<String>,
    pub compilation_time_ms: Option<i32>,
    pub code_size_bytes: Option<i32>,
    pub node_count: Option<i32>,
    pub edge_count: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// Upload log entry
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadLog {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub board_type: Option<String>,
    pub port: Option<String>,
    pub status: String, // 'success', 'error'
    pub error_message: Option<String>,
    pub upload_time_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// System metrics for a specific date
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemMetrics {
    pub id: Uuid,
    pub metric_date: NaiveDate,
    pub total_users: i32,
    pub active_users: i32,
    pub new_users: i32,
    pub total_projects: i32,
    pub new_projects: i32,
    pub public_projects: i32,
    pub total_compilations: i32,
    pub successful_compilations: i32,
    pub total_uploads: i32,
    pub successful_uploads: i32,
    pub total_views: i32,
    pub total_likes: i32,
    pub total_comments: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Admin dashboard overview metrics
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DashboardMetrics {
    pub total_users: i64,
    pub total_projects: i64,
    pub public_projects: i64,
    pub private_projects: i64,
    pub total_organizations: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_comments: i64,
    pub total_compilations: i64,
    pub successful_compilations: i64,
    pub total_uploads: i64,
    pub successful_uploads: i64,
    pub featured_projects: i64,
}

/// Time series data point
#[derive(Debug, Clone, Serialize)]
pub struct TimeSeriesPoint {
    pub date: NaiveDate,
    pub value: i32,
}

/// Metrics over time (for charts)
#[derive(Debug, Clone, Serialize)]
pub struct MetricsTimeSeries {
    pub users: Vec<TimeSeriesPoint>,
    pub projects: Vec<TimeSeriesPoint>,
    pub compilations: Vec<TimeSeriesPoint>,
    pub uploads: Vec<TimeSeriesPoint>,
}

/// Request to log a compilation
#[derive(Debug, Clone, Deserialize)]
pub struct LogCompilationRequest {
    pub project_id: Uuid,
    pub status: String,
    pub error_message: Option<String>,
    pub compilation_time_ms: Option<i32>,
    pub code_size_bytes: Option<i32>,
    pub node_count: Option<i32>,
    pub edge_count: Option<i32>,
}

/// Request to log an upload
#[derive(Debug, Clone, Deserialize)]
pub struct LogUploadRequest {
    pub project_id: Uuid,
    pub board_type: Option<String>,
    pub port: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub upload_time_ms: Option<i32>,
}

/// Top projects by metric
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TopProject {
    pub id: Uuid,
    pub title: String,
    pub author_email: String,
    pub value: i32, // view_count, like_count, etc.
    pub created_at: DateTime<Utc>,
}

/// User activity statistics
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct UserActivity {
    pub user_id: Uuid,
    pub user_email: String,
    pub user_name: Option<String>,
    pub project_count: i64,
    pub total_views: i64,
    pub total_likes: i64,
    pub total_compilations: i64,
}

/// Category distribution
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct CategoryStats {
    pub category: String,
    pub count: i64,
}

/// Difficulty distribution
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DifficultyStats {
    pub difficulty: String,
    pub count: i64,
}
