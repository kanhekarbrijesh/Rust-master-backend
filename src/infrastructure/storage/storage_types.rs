use async_trait::async_trait;

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

/// Every backend (Local, S3, R2) implements this trait.
#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn store_file(&self, input: StoreFileInput) -> Result<StoreFileResult, AppError>;
    async fn get_file_url(&self, key: &str) -> Result<String, AppError>;
    async fn delete_file(&self, key: &str) -> Result<(), AppError>;
    async fn file_exists(&self, key: &str) -> Result<bool, AppError>;
}

/// Generate a unique storage key: {directory}/{uuid}-{sanitized_file_name}
pub fn build_storage_key(directory: &str, file_name: &str) -> String {
    let sanitized: String = file_name
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect();
    let uuid = uuid::Uuid::new_v4();
    format!("{}/{}-{}", directory.trim_end_matches('/'), uuid, sanitized)
}

pub fn file_name_from_key(key: &str) -> Option<&str> {
    key.rsplit('/').next()
}
