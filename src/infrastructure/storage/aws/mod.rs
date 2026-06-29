// ─── AWS S3 STORAGE PROVIDER ────────────────────────────────────────────────
//
// Implements the StorageProvider trait using the AWS S3 SDK.
//
// Required env vars (see .env):
//   AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY, AWS_REGION, AWS_S3_BUCKET
// ============================================================================

use async_trait::async_trait;
use aws_sdk_s3::primitives::ByteStream;
use tracing::{error, info};

use crate::{
    _utils::app_error::AppError,
    infrastructure::storage::storage_types::{
        PresignedUploadConfig, PresignedUploadResult, StorageProvider, StoreFileInput,
        StoreFileResult, build_storage_key,
    },
};

/// AWS S3 storage provider.
pub struct AwsS3Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    region: String,
}

impl AwsS3Storage {
    /// Create a new `AwsS3Storage` provider.
    ///
    /// * `bucket` — S3 bucket name
    /// * `region` — AWS region (e.g. "ap-south-1")
    pub async fn new(bucket: &str, region: &str) -> Self {
        let region_owned = region.to_string();
        let config = aws_config::from_env()
            .region(aws_sdk_s3::config::Region::new(region_owned))
            .load()
            .await;

        let client = aws_sdk_s3::Client::new(&config);

        info!(
            bucket = %bucket,
            region = %region,
            "AwsS3Storage initialised"
        );

        Self {
            client,
            bucket: bucket.to_string(),
            region: region.to_string(),
        }
    }
}

#[async_trait]
impl StorageProvider for AwsS3Storage {
    async fn store_file(&self, input: StoreFileInput) -> Result<StoreFileResult, AppError> {
        let key = build_storage_key(&input.directory, &input.file_name);

        let body = ByteStream::from(input.buffer.clone());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .body(body)
            .content_type(&input.content_type)
            .send()
            .await
            .map_err(|e| {
                let service_err = e.into_service_error();
                let err_msg = format!("{} (code: {:?})", service_err, service_err.meta().code());
                error!(key = %key, bucket = %self.bucket, "S3 store_file failed: {err_msg}");
                AppError::Internal(format!("S3 storage error: {err_msg}"))
            })?;

        let size = input.buffer.len() as u64;

        // Standard S3 URL format: https://{bucket}.s3.{region}.amazonaws.com/{key}
        let url = format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, key
        );

        info!(key = %key, size = size, "File stored in S3");

        Ok(StoreFileResult {
            key,
            url,
            content_type: input.content_type,
            size,
        })
    }

    async fn get_file_url(&self, key: &str) -> Result<String, AppError> {
        Ok(format!(
            "https://{}.s3.{}.amazonaws.com/{}",
            self.bucket, self.region, key
        ))
    }

    async fn delete_file(&self, key: &str) -> Result<(), AppError> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let service_err = e.into_service_error();
                let err_msg = format!("{} (code: {:?})", service_err, service_err.meta().code());
                error!(key = %key, bucket = %self.bucket, "S3 delete_file failed: {err_msg}");
                AppError::Internal(format!("S3 storage error: {err_msg}"))
            })?;

        info!(key = %key, "File deleted from S3");
        Ok(())
    }

    async fn file_exists(&self, key: &str) -> Result<bool, AppError> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_not_found() || service_err.meta().code() == Some("NotFound") {
                    Ok(false)
                } else {
                    error!(key = %key, "S3 file_exists check failed");
                    Err(AppError::Internal(format!(
                        "S3 storage error: {service_err}"
                    )))
                }
            }
        }
    }

    async fn presigned_upload_url(
        &self,
        config: PresignedUploadConfig,
    ) -> Result<PresignedUploadResult, AppError> {
        let key = build_storage_key(&config.directory, &config.file_name);
        let expires = std::time::Duration::from_secs(config.expires_in_secs);

        let expires_cfg = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(expires)
            .build()
            .map_err(|e| AppError::Internal(format!("Presigning config error: {e}")))?;

        let presigned_req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&key)
            .content_type(&config.content_type)
            .presigned(expires_cfg)
            .await
            .map_err(|e| {
                error!(key = %key, bucket = %self.bucket, "S3 presigned URL failed: {e}");
                AppError::Internal(format!("S3 presigned URL error: {e}"))
            })?;

        info!(key = %key, expires_secs = %config.expires_in_secs, "S3 presigned upload URL generated");

        Ok(PresignedUploadResult {
            url: presigned_req.uri().to_string(),
            key,
            method: "PUT".to_string(),
            headers: Some(vec![("Content-Type".to_string(), config.content_type)]),
        })
    }
}
