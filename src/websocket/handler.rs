//! WebSocket connection handler and room management.

use std::collections::HashMap;
use std::sync::Arc;

use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use uuid::Uuid;

use super::messages::{ClientMessage, RoomUser, ServerMessage};

/// Type alias for the sender half of a WebSocket connection
pub type WsSender = mpsc::UnboundedSender<ServerMessage>;

/// Represents a connected user
#[derive(Debug, Clone)]
pub struct ConnectedUser {
    pub user_id: Uuid,
    pub user_name: String,
    pub avatar_url: Option<String>,
    pub sender: WsSender,
}

/// Manages all project collaboration rooms
#[derive(Debug, Default)]
pub struct RoomManager {
    /// Map of project_id -> connected users
    rooms: DashMap<Uuid, HashMap<Uuid, ConnectedUser>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: DashMap::new(),
        }
    }
    
    /// Add a user to a project room
    pub fn join_room(&self, project_id: Uuid, user: ConnectedUser) {
        let user_id = user.user_id;
        let user_name = user.user_name.clone();
        
        // Add user to room
        self.rooms
            .entry(project_id)
            .or_insert_with(HashMap::new)
            .insert(user_id, user);
        
        // Broadcast user joined to other users in the room
        self.broadcast_to_room_except(
            project_id,
            user_id,
            ServerMessage::user_joined(project_id, user_id, user_name),
        );
        
        // Send current room users to the new user
        if let Some(room) = self.rooms.get(&project_id) {
            let users: Vec<RoomUser> = room
                .iter()
                .map(|(_, u)| RoomUser {
                    user_id: u.user_id,
                    user_name: u.user_name.clone(),
                    avatar_url: u.avatar_url.clone(),
                })
                .collect();
            
            if let Some(user) = room.get(&user_id) {
                let _ = user.sender.send(ServerMessage::RoomUsers {
                    project_id,
                    users,
                });
            }
        }
    }
    
    /// Remove a user from a project room
    pub fn leave_room(&self, project_id: Uuid, user_id: Uuid) -> Option<String> {
        let mut user_name = None;
        
        // Remove user from room
        if let Some(mut room) = self.rooms.get_mut(&project_id) {
            if let Some(user) = room.remove(&user_id) {
                user_name = Some(user.user_name.clone());
            }
            
            // Remove room if empty
            if room.is_empty() {
                drop(room);
                self.rooms.remove(&project_id);
            }
        }
        
        // Broadcast user left to remaining users
        if let Some(name) = &user_name {
            self.broadcast_to_room(
                project_id,
                ServerMessage::user_left(project_id, user_id, name.clone()),
            );
        }
        
        user_name
    }
    
    /// Broadcast a message to all users in a room
    pub fn broadcast_to_room(&self, project_id: Uuid, message: ServerMessage) {
        if let Some(room) = self.rooms.get(&project_id) {
            for (_, user) in room.iter() {
                if let Err(e) = user.sender.send(message.clone()) {
                    warn!("Failed to send message to user {}: {}", user.user_id, e);
                }
            }
        }
    }
    
    /// Broadcast a message to all users in a room except one
    pub fn broadcast_to_room_except(
        &self,
        project_id: Uuid,
        except_user_id: Uuid,
        message: ServerMessage,
    ) {
        if let Some(room) = self.rooms.get(&project_id) {
            for (user_id, user) in room.iter() {
                if *user_id != except_user_id {
                    if let Err(e) = user.sender.send(message.clone()) {
                        warn!("Failed to send message to user {}: {}", user_id, e);
                    }
                }
            }
        }
    }
    
    /// Get the number of users in a room
    pub fn room_size(&self, project_id: Uuid) -> usize {
        self.rooms
            .get(&project_id)
            .map(|room| room.len())
            .unwrap_or(0)
    }
    
    /// Get all rooms and their sizes
    pub fn get_room_stats(&self) -> Vec<(Uuid, usize)> {
        self.rooms
            .iter()
            .map(|entry| (*entry.key(), entry.value().len()))
            .collect()
    }
}

/// Global room manager instance
pub static ROOM_MANAGER: once_cell::sync::Lazy<Arc<RoomManager>> =
    once_cell::sync::Lazy::new(|| Arc::new(RoomManager::new()));

/// Get the global room manager
pub fn room_manager() -> Arc<RoomManager> {
    ROOM_MANAGER.clone()
}

/// Handle an incoming client message
pub fn handle_client_message(
    project_id: Uuid,
    user_id: Uuid,
    user_name: &str,
    message: ClientMessage,
    sender: &WsSender,
) {
    let manager = room_manager();
    
    match message {
        ClientMessage::Ping => {
            let _ = sender.send(ServerMessage::Pong);
        }
        
        ClientMessage::Subscribe { project_id: sub_id } => {
            // Already subscribed via the URL path
            let _ = sender.send(ServerMessage::Subscribed { project_id: sub_id });
        }
        
        ClientMessage::Unsubscribe { project_id: unsub_id } => {
            manager.leave_room(unsub_id, user_id);
            let _ = sender.send(ServerMessage::Unsubscribed { project_id: unsub_id });
        }
        
        ClientMessage::ProjectUpdated { project_id, nodes, edges } => {
            // Broadcast update to other users in the room
            manager.broadcast_to_room_except(
                project_id,
                user_id,
                ServerMessage::project_updated(project_id, user_id, nodes, edges),
            );
        }
        
        ClientMessage::CursorMove { project_id, x, y } => {
            // Broadcast cursor position to other users
            manager.broadcast_to_room_except(
                project_id,
                user_id,
                ServerMessage::CursorMove {
                    project_id,
                    user_id,
                    user_name: user_name.to_string(),
                    x,
                    y,
                },
            );
        }
    }
}
