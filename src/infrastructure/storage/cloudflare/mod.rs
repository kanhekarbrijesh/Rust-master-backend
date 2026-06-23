use async_trait::async_trait;

use crate::_utils::app_error::AppError;
use crate::infrastructure::storage::storage_types::{StorageProvider, StoreFileInput, StoreFileResult};

/// Cloudflare R2 (S3-compatible) storage provider.
///
/// Uses the same S3 API as AWS — add `aws-sdk-s3` to Cargo.toml and configure
/// the R2 endpoint, access key, and secret key in your environment / config.
///
/// Reference implementation (Node.js) is available under `references/cloudflareUtils/`.
#[allow(dead_code)]
pub struct CloudflareR2Storage {
    _endpoint: String,
    _bucket: String,
}

impl CloudflareR2Storage {
    pub fn new(endpoint: &str, bucket: &str) -> Self {
        Self {
            _endpoint: endpoint.to_string(),
            _bucket: bucket.to_string(),
        }
    }
}

#[async_trait]
impl StorageProvider for CloudflareR2Storage {
    async fn store_file(&self, _input: StoreFileInput) -> Result<StoreFileResult, AppError> {
        Err(AppError::Internal("CloudflareR2Storage not yet implemented — add aws-sdk-s3 to Cargo.toml".into()))
    }

    async fn get_file_url(&self, _key: &str) -> Result<String, AppError> {
        Err(AppError::Internal("CloudflareR2Storage not yet implemented".into()))
    }

    async fn delete_file(&self, _key: &str) -> Result<(), AppError> {
        Err(AppError::Internal("CloudflareR2Storage not yet implemented".into()))
    }

    async fn file_exists(&self, _key: &str) -> Result<bool, AppError> {
        Err(AppError::Internal("CloudflareR2Storage not yet implemented".into()))
    }
}
