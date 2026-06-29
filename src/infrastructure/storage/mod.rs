pub mod aws;
pub mod cloudflare;
pub mod file_preprocess;
pub mod image_preprocessing;
pub mod localstorage;
pub mod storage_types;
pub mod storage_util;

// ─── RE-EXPORTS (convenience) ────────────────────────────────────────────────
pub use aws::AwsS3Storage;
pub use cloudflare::CloudflareR2Storage;
pub use localstorage::LocalStorage;
pub use storage_types::*;

// ─── RE-EXPORT preprocessors ─────────────────────────────────────────────────
pub use file_preprocess::file_preprocess_aws_lambda::AwsLambdaFilePreprocessor;
pub use file_preprocess::file_preprocess_cf_worker::CfWorkerFilePreprocessor;
pub use file_preprocess::file_preprocess_local::LocalFilePreprocessor;
pub use file_preprocess::{FilePreprocessor, PreprocessingConfig, PreprocessingResult};
