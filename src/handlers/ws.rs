//! WebSocket handlers.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::db::DbPool;
use crate::services::{auth_service, collaboration_service, project_service};
use crate::utils::jwt::verify_access_token;
use crate::websocket::{
    handler::{handle_client_message, room_manager, ConnectedUser},
    messages::{ClientMessage, ServerMessage},
};

/// Query parameters for WebSocket connection
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    /// JWT access token for authentication
    token: String,
}

/// Handle WebSocket connection for project collaboration
pub async fn project_websocket(
    ws: WebSocketUpgrade,
    State(pool): State<DbPool>,
    Path(project_id): Path<Uuid>,
    Query(query): Query<WsQuery>,
) -> Response {
    // Verify JWT token
    let claims = match verify_access_token(&query.token) {
        Ok(claims) => claims,
        Err(e) => {
            error!("WebSocket auth failed: {:?}", e);
            return ws.on_upgrade(|mut socket| async move {
                let _ = socket
                    .send(Message::Text(
                        serde_json::to_string(&ServerMessage::error("Authentication failed"))
                            .unwrap(),
                    ))
                    .await;
                let _ = socket.close().await;
            });
        }
    };
    
    let user_id = claims.sub;
    let user_email = claims.email.clone();
    
    // Clone pool for the async block
    let pool_clone = pool.clone();
    
    ws.on_upgrade(move |socket| async move {
        handle_socket(socket, pool_clone, project_id, user_id, user_email).await
    })
}

/// Handle the WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    pool: DbPool,
    project_id: Uuid,
    user_id: Uuid,
    _user_email: String,
) {
    // Check if this is a temporary collaboration session (temp-xxx)
    // Temporary sessions don't require project to exist in database
    let project_id_str = project_id.to_string();
    let is_temporary = project_id_str.starts_with("00000000-0000-0000-0000-");
    
    if !is_temporary {
        // Verify project exists and user has access
        let project = match project_service::get_project_by_id(&pool, project_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("WebSocket: Project not found: {:?}", e);
                return;
            }
        };
        
        // Check if user can access the project (owner, collaborator, or public)
        if !project.is_public {
            // Check if user is owner or collaborator using the collaboration service
            let can_access = match collaboration_service::can_user_access_project(&pool, project_id, user_id).await {
                Ok(access) => access,
                Err(e) => {
                    // If the function fails (table doesn't exist), fall back to owner check
                    warn!("WebSocket: can_user_access_project failed: {:?}, falling back to owner check", e);
                    project.user_id == user_id
                }
            };
            
            if !can_access {
                error!("WebSocket: User {} cannot access project {}", user_id, project_id);
                return;
            }
        }
    }
    
    // Get user info
    let user = match auth_service::get_current_user(&pool, user_id).await {
        Ok(u) => u,
        Err(e) => {
            error!("WebSocket: User not found: {:?}", e);
            return;
        }
    };
    
    let user_name = user.full_name.clone();
    let avatar_url = user.avatar_url.clone();
    
    info!(
        "WebSocket: User {} ({}) connected to project {}",
        user_id, user_name, project_id
    );
    
    // Split socket into sender and receiver
    let (mut ws_sender, mut ws_receiver) = socket.split();
    
    // Create channel for sending messages to this client
    let (tx, mut rx) = mpsc::unbounded_channel::<ServerMessage>();
    
    // Register user in the room
    let manager = room_manager();
    manager.join_room(
        project_id,
        ConnectedUser {
            user_id,
            user_name: user_name.clone(),
            avatar_url,
            sender: tx.clone(),
        },
    );
    
    // Task to forward messages from channel to WebSocket
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match serde_json::to_string(&msg) {
                Ok(text) => {
                    if ws_sender.send(Message::Text(text)).await.is_err() {
                        break;
                    }
                }
                Err(e) => {
                    error!("Failed to serialize message: {:?}", e);
                }
            }
        }
    });
    
    // Task to handle incoming messages
    let user_name_clone = user_name.clone();
    let tx_clone = tx.clone();
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_receiver.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => {
                            handle_client_message(
                                project_id,
                                user_id,
                                &user_name_clone,
                                client_msg,
                                &tx_clone,
                            );
                        }
                        Err(e) => {
                            warn!("Invalid WebSocket message: {:?}", e);
                            let _ = tx_clone.send(ServerMessage::error(format!(
                                "Invalid message format: {}",
                                e
                            )));
                        }
                    }
                }
                Message::Ping(data) => {
                    // Axum handles pong automatically
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });
    
    // Wait for either task to complete (connection closed)
    tokio::select! {
        _ = send_task => {},
        _ = recv_task => {},
    }
    
    // Clean up: remove user from room
    manager.leave_room(project_id, user_id);
    
    info!(
        "WebSocket: User {} ({}) disconnected from project {}",
        user_id, user_name, project_id
    );
}
