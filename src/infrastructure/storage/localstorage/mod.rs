use async_trait::async_trait;
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::_utils::app_error::AppError;
use crate::infrastructure::storage::storage_types::{
    PresignedUploadConfig, PresignedUploadResult, StorageProvider, StoreFileInput, StoreFileResult,
    build_storage_key,
};

/// Local file-system storage provider.
///
/// Files are stored under `{base_path}/{storage_key}`.
/// The `url` in `StoreFileResult` is a relative path from the server root
/// (e.g. `/uploads/gallery/uuid-photo.png`).
pub struct LocalStorage {
    base_path: String,
    serve_prefix: String,
}

impl LocalStorage {
    /// Create a new `LocalStorage` provider.
    ///
    /// * `base_path` — Absolute or relative filesystem path (e.g. `"./uploads"`).
    /// * `serve_prefix` — URL prefix used to construct the public URL
    ///   (e.g. `"/uploads"`).
    pub fn new(base_path: &str, serve_prefix: &str) -> Self {
        Self {
            base_path: base_path.trim_end_matches('/').to_string(),
            serve_prefix: serve_prefix.trim_end_matches('/').to_string(),
        }
    }
}

#[async_trait]
impl StorageProvider for LocalStorage {
    async fn store_file(&self, input: StoreFileInput) -> Result<StoreFileResult, AppError> {
        let key = build_storage_key(&input.directory, &input.file_name);

        // Full filesystem path: ./uploads/gallery/uuid-photo.png
        let full_path = format!("{}/{}", self.base_path, key);

        // Ensure the parent directory exists
        if let Some(parent) = std::path::Path::new(&full_path).parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        // Write the file
        let mut file = fs::File::create(&full_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to create file: {}", e)))?;

        file.write_all(&input.buffer)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

        let size = input.buffer.len() as u64;

        // Public URL (relative)  e.g. /uploads/gallery/uuid-photo.png
        let url = format!("{}/{}", self.serve_prefix, key);

        Ok(StoreFileResult {
            key,
            url,
            content_type: input.content_type,
            size,
        })
    }

    async fn get_file_url(&self, key: &str) -> Result<String, AppError> {
        Ok(format!("{}/{}", self.serve_prefix, key))
    }

    async fn delete_file(&self, key: &str) -> Result<(), AppError> {
        let full_path = format!("{}/{}", self.base_path, key);

        fs::remove_file(&full_path)
            .await
            .map_err(|e| AppError::Internal(format!("Failed to delete file: {}", e)))?;

        Ok(())
    }

    async fn file_exists(&self, key: &str) -> Result<bool, AppError> {
        let full_path = format!("{}/{}", self.base_path, key);
        Ok(std::path::Path::new(&full_path).exists())
    }

    async fn presigned_upload_url(
        &self,
        _config: PresignedUploadConfig,
    ) -> Result<PresignedUploadResult, AppError> {
        Err(AppError::BadRequest(
            "Presigned URLs are not supported by the local filesystem storage provider. \
             Use a cloud storage provider (AWS S3 or Cloudflare R2) or fall back to \
             direct upload via `store_file()`."
                .into(),
        ))
    }
}
