// ─── CLOUDFLARE R2 STORAGE PROVIDER ─────────────────────────────────────────
//
// Implements the StorageProvider trait using the S3-compatible API.
// Cloudflare R2 uses the same S3 API as AWS, so we use aws-sdk-s3 with a
// custom endpoint.
//
// Required env vars (see .env):
//   R2_ACCESS_KEY, R2_SECRET_KEY, R2_ENDPOINT, R2_BUCKET, R2_KEY_PREFIX
// ============================================================================

use async_trait::async_trait;
use aws_sdk_s3::{config::Credentials, primitives::ByteStream};
use tracing::{error, info};

use crate::{
    _utils::app_error::AppError,
    infrastructure::storage::storage_types::{
        PresignedUploadConfig, PresignedUploadResult, StorageProvider, StoreFileInput,
        StoreFileResult, build_storage_key,
    },
};

/// Cloudflare R2 (S3-compatible) storage provider.
pub struct CloudflareR2Storage {
    client: aws_sdk_s3::Client,
    bucket: String,
    key_prefix: String,
    public_url_base: String,
    endpoint: String,
}

impl CloudflareR2Storage {
    /// Create a new `CloudflareR2Storage` provider.
    ///
    /// * `access_key` — R2 Access Key ID
    /// * `secret_key` — R2 Secret Access Key
    /// * `endpoint` — R2 endpoint URL
    /// * `bucket` — R2 bucket name
    /// * `key_prefix` — Optional prefix for all keys (e.g. "testing")
    /// * `public_url_base` — Public base URL for served files
    pub async fn new(
        access_key: &str,
        secret_key: &str,
        endpoint: &str,
        bucket: &str,
        key_prefix: &str,
        public_url_base: &str,
    ) -> Self {
        let credentials = Credentials::new(access_key, secret_key, None, None, "r2");

        let config = aws_config::from_env()
            .credentials_provider(credentials)
            .endpoint_url(endpoint)
            .region(aws_sdk_s3::config::Region::new("auto"))
            .load()
            .await;

        let client = aws_sdk_s3::Client::new(&config);

        info!(
            bucket = %bucket,
            endpoint = %endpoint,
            "CloudflareR2Storage initialised"
        );

        Self {
            client,
            bucket: bucket.to_string(),
            key_prefix: key_prefix.trim_end_matches('/').to_string(),
            public_url_base: public_url_base.trim_end_matches('/').to_string(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
        }
    }

    fn build_full_key(&self, raw_key: &str) -> String {
        if self.key_prefix.is_empty() {
            raw_key.to_string()
        } else {
            format!("{}/{}", self.key_prefix, raw_key)
        }
    }
}

#[async_trait]
impl StorageProvider for CloudflareR2Storage {
    async fn store_file(&self, input: StoreFileInput) -> Result<StoreFileResult, AppError> {
        let raw_key = build_storage_key(&input.directory, &input.file_name);
        let full_key = self.build_full_key(&raw_key);

        let body = ByteStream::from(input.buffer.clone());

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .body(body)
            .content_type(&input.content_type)
            .send()
            .await
            .map_err(|e| {
                error!(key = %full_key, bucket = %self.bucket, "R2 store_file failed: {e}");
                AppError::Internal(format!("R2 storage error: {e}"))
            })?;

        let size = input.buffer.len() as u64;

        let url = if self.public_url_base.is_empty() {
            format!("{}/{}", self.endpoint, full_key)
        } else {
            format!("{}/{}", self.public_url_base, raw_key)
        };

        info!(key = %full_key, size = size, "File stored in R2");

        Ok(StoreFileResult {
            key: raw_key,
            url,
            content_type: input.content_type,
            size,
        })
    }

    async fn get_file_url(&self, key: &str) -> Result<String, AppError> {
        if self.public_url_base.is_empty() {
            Ok(format!("{}/{}", self.endpoint, self.build_full_key(key)))
        } else {
            Ok(format!("{}/{}", self.public_url_base, key))
        }
    }

    async fn delete_file(&self, key: &str) -> Result<(), AppError> {
        let full_key = self.build_full_key(key);

        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
            .map_err(|e| {
                error!(key = %full_key, bucket = %self.bucket, "R2 delete_file failed: {e}");
                AppError::Internal(format!("R2 storage error: {e}"))
            })?;

        info!(key = %full_key, "File deleted from R2");
        Ok(())
    }

    async fn file_exists(&self, key: &str) -> Result<bool, AppError> {
        let full_key = self.build_full_key(key);

        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(err) => {
                let service_err = err.into_service_error();
                if service_err.is_not_found() || service_err.meta().code() == Some("NotFound") {
                    Ok(false)
                } else {
                    error!(key = %full_key, "R2 file_exists check failed");
                    Err(AppError::Internal(format!(
                        "R2 storage error: {service_err}"
                    )))
                }
            }
        }
    }

    async fn presigned_upload_url(
        &self,
        config: PresignedUploadConfig,
    ) -> Result<PresignedUploadResult, AppError> {
        let raw_key = build_storage_key(&config.directory, &config.file_name);
        let full_key = self.build_full_key(&raw_key);
        let expires = std::time::Duration::from_secs(config.expires_in_secs);

        let expires_cfg = aws_sdk_s3::presigning::PresigningConfig::builder()
            .expires_in(expires)
            .build()
            .map_err(|e| AppError::Internal(format!("Presigning config error: {e}")))?;

        let presigned_req = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(&full_key)
            .content_type(&config.content_type)
            .presigned(expires_cfg)
            .await
            .map_err(|e| {
                error!(key = %full_key, bucket = %self.bucket, "R2 presigned URL failed: {e}");
                AppError::Internal(format!("R2 presigned URL error: {e}"))
            })?;

        info!(key = %raw_key, expires_secs = %config.expires_in_secs, "R2 presigned upload URL generated");

        Ok(PresignedUploadResult {
            url: presigned_req.uri().to_string(),
            key: raw_key,
            method: "PUT".to_string(),
            headers: Some(vec![("Content-Type".to_string(), config.content_type)]),
        })
    }
}
