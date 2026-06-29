use async_trait::async_trait;
use serde::Serialize;

use crate::_utils::app_error::AppError;

/// Input for storing a file via any provider
#[derive(Debug, Clone)]
pub struct StoreFileInput {
    pub buffer: Vec<u8>,
    pub file_name: String,
    pub content_type: String,
    pub directory: String,
}

/// Result returned after a successful store operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StoreFileResult {
    pub key: String,
    pub url: String,
    pub content_type: String,
    pub size: u64,
}

/// Configuration for generating a presigned upload URL.
#[derive(Debug, Clone)]
pub struct PresignedUploadConfig {
    /// MIME type of the file (e.g. "image/jpeg").
    pub content_type: String,
    /// Original file name (used to generate a unique storage key).
    pub file_name: String,
    /// Target directory/prefix in the bucket (e.g. "gallery").
    pub directory: String,
    /// How long the URL is valid, in seconds (default: 3600).
    pub expires_in_secs: u64,
}

impl Default for PresignedUploadConfig {
    fn default() -> Self {
        Self {
            content_type: String::new(),
            file_name: String::new(),
            directory: "uploads".to_string(),
            expires_in_secs: 3600,
        }
    }
}

/// Result from generating a presigned upload URL.
#[derive(Debug, Clone, Serialize)]
pub struct PresignedUploadResult {
    /// The presigned URL the frontend uses to upload the file directly.
    pub url: String,
    /// The storage key reserved for this upload.
    pub key: String,
    /// The HTTP method for the upload (always "PUT").
    pub method: String,
    /// Additional required headers (e.g. Content-Type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<(String, String)>>,
}

/// Every backend (Local, S3, R2) implements this trait.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn store_file(&self, input: StoreFileInput) -> Result<StoreFileResult, AppError>;
    async fn get_file_url(&self, key: &str) -> Result<String, AppError>;
    async fn delete_file(&self, key: &str) -> Result<(), AppError>;
    async fn file_exists(&self, key: &str) -> Result<bool, AppError>;

    /// Generate a presigned upload URL for direct browser-to-cloud uploads.
    ///
    /// **Supported by:** AWS S3, Cloudflare R2
    /// **Not supported by:** LocalStorage (returns `AppError::BadRequest`)
    ///
    /// Controllers should fall back to `store_file()` when this returns an error.
    async fn presigned_upload_url(
        &self,
        config: PresignedUploadConfig,
    ) -> Result<PresignedUploadResult, AppError>;
}

/// Generate a unique storage key: {directory}/{uuid}-{sanitized_file_name}
pub fn build_storage_key(directory: &str, file_name: &str) -> String {
    let sanitized: String = file_name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let uuid = uuid::Uuid::new_v4();
    format!("{}/{}-{}", directory.trim_end_matches('/'), uuid, sanitized)
}

pub fn file_name_from_key(key: &str) -> Option<&str> {
    key.rsplit('/').next()
}
