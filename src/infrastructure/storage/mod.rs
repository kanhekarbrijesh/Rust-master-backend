pub mod aws;
pub mod cloudflare;
pub mod image_preprocessing;
pub mod localstorage;
pub mod storage_types;
pub mod storage_util;

// ─── RE-EXPORTS (convenience) ────────────────────────────────────────────────
pub use aws::AwsS3Storage;
pub use cloudflare::CloudflareR2Storage;
pub use localstorage::LocalStorage;
pub use storage_types::*;
