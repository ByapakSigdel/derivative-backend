//! File storage utilities for handling uploads.

use std::path::{Path, PathBuf};

use chrono::Utc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::config::CONFIG;
use crate::errors::{AppError, AppResult};

/// Allowed image MIME types
const ALLOWED_IMAGE_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/webp",
    "image/gif",
];

/// Allowed asset MIME types (images + more)
const ALLOWED_ASSET_TYPES: &[&str] = &[
    "image/jpeg",
    "image/jpg",
    "image/png",
    "image/webp",
    "image/gif",
    "image/svg+xml",
    "application/json",
    "text/plain",
];

/// File type category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileCategory {
    Avatar,
    ProjectAsset,
}

impl FileCategory {
    /// Get the subdirectory for this file category
    pub fn subdirectory(&self) -> &'static str {
        match self {
            FileCategory::Avatar => "avatars",
            FileCategory::ProjectAsset => "project_assets",
        }
    }
    
    /// Get the URL path prefix for this file category
    pub fn url_prefix(&self) -> &'static str {
        match self {
            FileCategory::Avatar => "/api/uploads/avatars",
            FileCategory::ProjectAsset => "/api/uploads/project-assets",
        }
    }
    
    /// Get allowed MIME types for this category
    pub fn allowed_types(&self) -> &[&str] {
        match self {
            FileCategory::Avatar => ALLOWED_IMAGE_TYPES,
            FileCategory::ProjectAsset => ALLOWED_ASSET_TYPES,
        }
    }
    
    /// Get maximum file size for this category
    pub fn max_size(&self) -> usize {
        match self {
            FileCategory::Avatar => CONFIG.max_avatar_size,
            FileCategory::ProjectAsset => CONFIG.max_asset_size,
        }
    }
}

/// Information about a stored file
#[derive(Debug, Clone)]
pub struct StoredFile {
    /// Unique filename
    pub filename: String,
    /// Full path on disk
    pub path: PathBuf,
    /// Public URL to access the file
    pub url: String,
    /// Original filename
    pub original_name: String,
    /// File size in bytes
    pub size: usize,
    /// MIME type
    pub content_type: String,
}

/// Validate file type
pub fn validate_file_type(content_type: &str, category: FileCategory) -> AppResult<()> {
    if !category.allowed_types().contains(&content_type) {
        return Err(AppError::InvalidFileType(format!(
            "File type '{}' is not allowed. Allowed types: {:?}",
            content_type,
            category.allowed_types()
        )));
    }
    Ok(())
}

/// Validate file size
pub fn validate_file_size(size: usize, category: FileCategory) -> AppResult<()> {
    if size > category.max_size() {
        return Err(AppError::FileTooLarge);
    }
    Ok(())
}

/// Generate a unique filename
pub fn generate_filename(owner_id: Uuid, original_name: &str) -> String {
    let timestamp = Utc::now().timestamp_millis();
    let extension = Path::new(original_name)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");
    
    format!("{}_{}.{}", owner_id, timestamp, extension)
}

/// Get the full storage path for a file
pub fn get_storage_path(category: FileCategory, filename: &str) -> PathBuf {
    PathBuf::from(&CONFIG.upload_dir)
        .join(category.subdirectory())
        .join(filename)
}

/// Get the public URL for a stored file
pub fn get_file_url(category: FileCategory, filename: &str) -> String {
    format!("{}/{}", category.url_prefix(), filename)
}

/// Ensure the upload directory exists
pub async fn ensure_upload_dirs() -> AppResult<()> {
    let avatars_dir = PathBuf::from(&CONFIG.upload_dir).join("avatars");
    let assets_dir = PathBuf::from(&CONFIG.upload_dir).join("project_assets");
    
    fs::create_dir_all(&avatars_dir)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create avatars directory: {}", e)))?;
    
    fs::create_dir_all(&assets_dir)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to create project_assets directory: {}", e)))?;
    
    Ok(())
}

/// Store a file to disk
pub async fn store_file(
    owner_id: Uuid,
    original_name: &str,
    content_type: &str,
    data: &[u8],
    category: FileCategory,
) -> AppResult<StoredFile> {
    // Validate file
    validate_file_type(content_type, category)?;
    validate_file_size(data.len(), category)?;
    
    // Generate unique filename and paths
    let filename = generate_filename(owner_id, original_name);
    let path = get_storage_path(category, &filename);
    let url = get_file_url(category, &filename);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| AppError::FileUpload(format!("Failed to create directory: {}", e)))?;
    }
    
    // Write file to disk
    let mut file = fs::File::create(&path)
        .await
        .map_err(|e| AppError::FileUpload(format!("Failed to create file: {}", e)))?;
    
    file.write_all(data)
        .await
        .map_err(|e| AppError::FileUpload(format!("Failed to write file: {}", e)))?;
    
    file.flush()
        .await
        .map_err(|e| AppError::FileUpload(format!("Failed to flush file: {}", e)))?;
    
    Ok(StoredFile {
        filename,
        path,
        url,
        original_name: original_name.to_string(),
        size: data.len(),
        content_type: content_type.to_string(),
    })
}

/// Delete a file from disk
pub async fn delete_file(category: FileCategory, filename: &str) -> AppResult<()> {
    let path = get_storage_path(category, filename);
    
    if path.exists() {
        fs::remove_file(&path)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to delete file: {}", e)))?;
    }
    
    Ok(())
}

/// Delete old file when replacing (e.g., updating avatar)
pub async fn delete_old_file_if_exists(url: Option<&str>, category: FileCategory) -> AppResult<()> {
    if let Some(url) = url {
        // Extract filename from URL
        if let Some(filename) = url.split('/').last() {
            delete_file(category, filename).await?;
        }
    }
    Ok(())
}

/// Get file extension from content type
pub fn extension_from_content_type(content_type: &str) -> &'static str {
    match content_type {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/svg+xml" => "svg",
        "application/json" => "json",
        "text/plain" => "txt",
        _ => "bin",
    }
}

/// Guess content type from filename
pub fn content_type_from_filename(filename: &str) -> String {
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_image_types() {
        assert!(validate_file_type("image/jpeg", FileCategory::Avatar).is_ok());
        assert!(validate_file_type("image/png", FileCategory::Avatar).is_ok());
        assert!(validate_file_type("image/webp", FileCategory::Avatar).is_ok());
        assert!(validate_file_type("application/pdf", FileCategory::Avatar).is_err());
    }
    
    #[test]
    fn test_generate_filename() {
        let owner_id = Uuid::new_v4();
        let filename = generate_filename(owner_id, "test.jpg");
        
        assert!(filename.starts_with(&owner_id.to_string()));
        assert!(filename.ends_with(".jpg"));
    }
    
    #[test]
    fn test_extension_from_content_type() {
        assert_eq!(extension_from_content_type("image/jpeg"), "jpg");
        assert_eq!(extension_from_content_type("image/png"), "png");
        assert_eq!(extension_from_content_type("unknown/type"), "bin");
    }
}
