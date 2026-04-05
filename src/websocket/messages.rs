//! WebSocket message types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Incoming WebSocket message from client
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Ping to keep connection alive
    Ping,
    
    /// Subscribe to project updates
    Subscribe { project_id: Uuid },
    
    /// Unsubscribe from project updates
    Unsubscribe { project_id: Uuid },
    
    /// Project was updated (nodes/edges changed)
    ProjectUpdated { 
        project_id: Uuid,
        nodes: Option<serde_json::Value>,
        edges: Option<serde_json::Value>,
    },
    
    /// Cursor position update (for collaborative editing)
    CursorMove {
        project_id: Uuid,
        x: f64,
        y: f64,
    },
}

/// Outgoing WebSocket message to client
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Pong response to ping
    Pong,
    
    /// Subscription confirmed
    Subscribed { project_id: Uuid },
    
    /// Unsubscription confirmed
    Unsubscribed { project_id: Uuid },
    
    /// Error message
    Error { message: String },
    
    /// User joined the project room
    UserJoined {
        project_id: Uuid,
        user_id: Uuid,
        user_name: String,
        timestamp: DateTime<Utc>,
    },
    
    /// User left the project room
    UserLeft {
        project_id: Uuid,
        user_id: Uuid,
        user_name: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Project was updated
    ProjectUpdated {
        project_id: Uuid,
        user_id: Uuid,
        timestamp: DateTime<Utc>,
        payload: ProjectUpdatePayload,
    },
    
    /// New comment was added
    CommentAdded {
        project_id: Uuid,
        user_id: Uuid,
        comment_id: Uuid,
        content: String,
        timestamp: DateTime<Utc>,
    },
    
    /// Project was liked
    LikeAdded {
        project_id: Uuid,
        user_id: Uuid,
        like_count: i32,
        timestamp: DateTime<Utc>,
    },
    
    /// Project was unliked
    LikeRemoved {
        project_id: Uuid,
        user_id: Uuid,
        like_count: i32,
        timestamp: DateTime<Utc>,
    },
    
    /// Current users in the room
    RoomUsers {
        project_id: Uuid,
        users: Vec<RoomUser>,
    },
    
    /// Cursor position update from another user
    CursorMove {
        project_id: Uuid,
        user_id: Uuid,
        user_name: String,
        x: f64,
        y: f64,
    },
}

/// Payload for project update events
#[derive(Debug, Clone, Serialize)]
pub struct ProjectUpdatePayload {
    pub nodes: Option<serde_json::Value>,
    pub edges: Option<serde_json::Value>,
}

/// User information for room participants
#[derive(Debug, Clone, Serialize)]
pub struct RoomUser {
    pub user_id: Uuid,
    pub user_name: String,
    pub avatar_url: Option<String>,
}

impl ServerMessage {
    /// Create an error message
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }
    
    /// Create a user joined message
    pub fn user_joined(project_id: Uuid, user_id: Uuid, user_name: String) -> Self {
        Self::UserJoined {
            project_id,
            user_id,
            user_name,
            timestamp: Utc::now(),
        }
    }
    
    /// Create a user left message
    pub fn user_left(project_id: Uuid, user_id: Uuid, user_name: String) -> Self {
        Self::UserLeft {
            project_id,
            user_id,
            user_name,
            timestamp: Utc::now(),
        }
    }
    
    /// Create a project updated message
    pub fn project_updated(
        project_id: Uuid,
        user_id: Uuid,
        nodes: Option<serde_json::Value>,
        edges: Option<serde_json::Value>,
    ) -> Self {
        Self::ProjectUpdated {
            project_id,
            user_id,
            timestamp: Utc::now(),
            payload: ProjectUpdatePayload { nodes, edges },
        }
    }
    
    /// Create a comment added message
    pub fn comment_added(
        project_id: Uuid,
        user_id: Uuid,
        comment_id: Uuid,
        content: String,
    ) -> Self {
        Self::CommentAdded {
            project_id,
            user_id,
            comment_id,
            content,
            timestamp: Utc::now(),
        }
    }
    
    /// Create a like added message
    pub fn like_added(project_id: Uuid, user_id: Uuid, like_count: i32) -> Self {
        Self::LikeAdded {
            project_id,
            user_id,
            like_count,
            timestamp: Utc::now(),
        }
    }
}
