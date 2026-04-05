//! User handlers.

use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;

use crate::db::DbPool;
use crate::errors::{AppError, AppResult};
use crate::middleware::auth::CurrentUser;
use crate::services::user_service;
use crate::utils::file_storage::{delete_old_file_if_exists, store_file, FileCategory};

/// Upload user avatar
pub async fn upload_avatar(
    State(pool): State<DbPool>,
    user: CurrentUser,
    mut multipart: Multipart,
) -> AppResult<Json<AvatarResponse>> {
    // Get the current user to check for existing avatar
    let current_user = user_service::get_user_by_id(&pool, user.id()).await?;
    
    // Process the multipart upload
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::FileUpload(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "avatar" || name == "file" {
            let filename = field
                .file_name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "avatar.jpg".to_string());
            
            let content_type = field
                .content_type()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());
            
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::FileUpload(e.to_string()))?;
            
            // Store the file
            let stored = store_file(
                user.id(),
                &filename,
                &content_type,
                &data,
                FileCategory::Avatar,
            )
            .await?;
            
            // Delete old avatar if exists
            delete_old_file_if_exists(current_user.avatar_url.as_deref(), FileCategory::Avatar)
                .await?;
            
            // Update user's avatar URL
            user_service::update_avatar(&pool, user.id(), &stored.url).await?;
            
            return Ok(Json(AvatarResponse {
                url: stored.url,
                filename: stored.filename,
            }));
        }
    }
    
    Err(AppError::BadRequest("No avatar file provided".to_string()))
}

#[derive(Serialize)]
pub struct AvatarResponse {
    url: String,
    filename: String,
}
