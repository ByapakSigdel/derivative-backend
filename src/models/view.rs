//! View model and related types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::net::IpAddr;
use uuid::Uuid;

/// Project view entity from the database
#[derive(Debug, Clone, Serialize, FromRow)]
pub struct ProjectView {
    pub id: Uuid,
    pub project_id: Uuid,
    pub user_id: Option<Uuid>,
    pub view_duration: Option<i32>,
    pub referrer: Option<String>,
    pub ip_address: Option<IpAddr>,
    pub viewed_at: DateTime<Utc>,
}

/// Request body for recording a view
#[derive(Debug, Deserialize)]
pub struct RecordViewRequest {
    pub view_duration: Option<i32>,
    pub referrer: Option<String>,
}

/// View response
#[derive(Debug, Serialize)]
pub struct ViewResponse {
    pub view_count: i32,
    pub recorded: bool,
}

/// View analytics for a project
#[derive(Debug, Serialize)]
pub struct ProjectViewAnalytics {
    pub total_views: i64,
    pub unique_viewers: i64,
    pub average_duration: Option<f64>,
    pub views_today: i64,
    pub views_this_week: i64,
    pub views_this_month: i64,
}

/// Daily view count
#[derive(Debug, Serialize, FromRow)]
pub struct DailyViewCount {
    pub date: DateTime<Utc>,
    pub view_count: i64,
}
