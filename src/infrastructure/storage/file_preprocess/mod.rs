// ─── FILE PREPROCESSING MODULE ──────────────────────────────────────────────
//
// This module provides a swappable file preprocessing architecture.
// Depending on where the code runs (local server, Cloudflare Worker,
// AWS Lambda), a different preprocessor implementation is used — all
// behind a common trait.
//
// **Why hardcoded swapping instead of env-based?**
// Cloudflare Workers and AWS Lambda are separate deployment targets with
// different runtime constraints.  The `image` crate with full WebP support
// works on native (local/Lambda) but NOT inside a Cloudflare Worker
// (Workers use a V8 isolate with limited WASM).  By swapping at code level
// you get compile-time validation that only the right dependencies are
// pulled in for each target.
//
// **How to swap:**
// ───────────────────────────────────────────────────────────────────────
// In `AppState::new()` (or the equivalent entry point), instantiate the
// preprocessor you want:
//
// ```rust
// use crate::infrastructure::storage::file_preprocess::{
//     FilePreprocessor,
//     file_preprocess_local::LocalFilePreprocessor,
// };
//
// let preprocessor: Arc<dyn FilePreprocessor> =
//     Arc::new(LocalFilePreprocessor::new());
// ```
//
// To switch to a Cloudflare‑compatible preprocessor:
// ```rust
// use crate::infrastructure::storage::file_preprocess::{
//     FilePreprocessor,
//     file_preprocess_cf_worker::CfWorkerFilePreprocessor,
// };
//
// let preprocessor: Arc<dyn FilePreprocessor> =
//     Arc::new(CfWorkerFilePreprocessor::new());
// ```
// ───────────────────────────────────────────────────────────────────────
//
// **Deployment targets:**
// | Target                | Preprocessor module         | Cargo feature |
// |-----------------------|-----------------------------|---------------|
// | Local (cargo run)     | file_preprocess_local       | default       |
// | Cloudflare Worker     | file_preprocess_cf_worker   | (separate JS) |
// | AWS Lambda            | file_preprocess_aws_lambda  | cfg flag      |
// ============================================================================

pub mod file_preprocess_aws_lambda;
pub mod file_preprocess_cf_worker;
pub mod file_preprocess_local;

use async_trait::async_trait;

use crate::_utils::app_error::AppError;

/// Configuration for how a file should be preprocessed.
#[derive(Debug, Clone)]
pub struct PreprocessingConfig {
    /// Maximum width/height dimension for image resize (0 = no resize).
    pub max_dimension: u32,
    /// Whether to convert images to WebP.
    pub convert_to_webp: bool,
    /// Allowed MIME types.  If empty, all types are accepted.
    pub allowed_mime_types: Vec<String>,
}

impl Default for PreprocessingConfig {
    fn default() -> Self {
        Self {
            max_dimension: 512,
            convert_to_webp: true,
            allowed_mime_types: vec![
                "image/jpeg".into(),
                "image/png".into(),
                "image/webp".into(),
                "image/gif".into(),
            ],
        }
    }
}

/// Result of a preprocessing operation.
#[derive(Debug, Clone)]
pub struct PreprocessingResult {
    /// The (potentially transformed) file bytes.
    pub buffer: Vec<u8>,
    /// The (potentially changed) content type (e.g. "image/webp").
    pub content_type: String,
    /// Original file name before processing (may be adjusted during processing).
    pub file_name: String,
}

/// Every file preprocessor implements this trait.
///
/// Implementations can be swapped at compile-time by choosing which
/// concrete struct to instantiate.
#[async_trait]
pub trait FilePreprocessor: Send + Sync {
    /// Validate MIME type and preprocess the file buffer.
    ///
    /// Returns a [`PreprocessingResult`] with the (possibly transformed)
    /// buffer, content type, and file name.
    fn preprocess(
        &self,
        buffer: &[u8],
        content_type: &str,
        file_name: &str,
    ) -> Result<PreprocessingResult, AppError>;
}

// ─── Convenience helper ─────────────────────────────────────────────────────

/// Validate the MIME type against the preprocessor's allowed list.
///
/// Every `FilePreprocessor` implementation should call this first.
pub fn validate_mime_type(content_type: &str, allowed: &[String]) -> Result<(), AppError> {
    if allowed.is_empty() {
        return Ok(()); // no restriction
    }
    if allowed.iter().any(|a| a == content_type) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Unsupported file type '{}'. Allowed: {}",
            content_type,
            allowed.join(", ")
        )))
    }
}

/// Strip the last file extension from a name, returning the base.
/// E.g. `"photo.jpeg"` → `"photo"`, `"photo"` → `None`.
pub fn strip_extension(file_name: &str) -> Option<&str> {
    let dot = file_name.rfind('.')?;
    if dot == 0 {
        None
    } else {
        Some(&file_name[..dot])
    }
}
