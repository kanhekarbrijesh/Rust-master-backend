use async_trait::async_trait;

use crate::_utils::app_error::AppError;
use crate::infrastructure::storage::storage_types::{StorageProvider, StoreFileInput, StoreFileResult};

/// AWS S3 storage provider.
///
/// Requires `aws-sdk-s3` and `aws-config` crate dependencies.
/// This is a placeholder / stub — wire it up once the SDK is added to Cargo.toml.
#[allow(dead_code)]
pub struct AwsS3Storage {
    _bucket: String,
    _region: String,
}

impl AwsS3Storage {
    pub fn new(bucket: &str, region: &str) -> Self {
        Self {
            _bucket: bucket.to_string(),
            _region: region.to_string(),
        }
    }
}

#[async_trait]
impl StorageProvider for AwsS3Storage {
    async fn store_file(&self, _input: StoreFileInput) -> Result<StoreFileResult, AppError> {
        Err(AppError::Internal("AwsS3Storage not yet implemented — add aws-sdk-s3 to Cargo.toml".into()))
    }

    async fn get_file_url(&self, _key: &str) -> Result<String, AppError> {
        Err(AppError::Internal("AwsS3Storage not yet implemented".into()))
    }

    async fn delete_file(&self, _key: &str) -> Result<(), AppError> {
        Err(AppError::Internal("AwsS3Storage not yet implemented".into()))
    }

    async fn file_exists(&self, _key: &str) -> Result<bool, AppError> {
        Err(AppError::Internal("AwsS3Storage not yet implemented".into()))
    }
}
