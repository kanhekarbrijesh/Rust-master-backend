// ─── IMAGE PREPROCESSING PIPELINE ─────────────────────────────────────────────
//
// **BACKWARD-COMPATIBILITY WRAPPER**
//
// This file now delegates to the modular `file_preprocess` architecture.
// Direct calls to `validate_mime_type()` and `preprocess_profile_image()`
// still work — they forward to a `LocalFilePreprocessor` under the hood.
//
// **New code should use the `FilePreprocessor` trait directly:**
// ```rust
// use crate::infrastructure::storage::file_preprocess::{
//     FilePreprocessor, file_preprocess_local::LocalFilePreprocessor,
// };
// let preprocessor = LocalFilePreprocessor::new();
// let result = preprocessor.preprocess(&buffer, "image/jpeg", "photo.jpg")?;
// ```
// ============================================================================

use std::sync::LazyLock;

use crate::_utils::app_error::AppError;

use super::file_preprocess::{
    FilePreprocessor, PreprocessingConfig, file_preprocess_local::LocalFilePreprocessor,
};

// ─── RE-EXPORT constants for backward compatibility ─────────────────────────

/// Allowed MIME types for image uploads.
pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Maximum dimension (width or height) for profile images.
/// Images larger than this will be resized proportionally.
pub const MAX_DIMENSION: u32 = 512;

static LEGACY_PREPROCESSOR: LazyLock<LocalFilePreprocessor> = LazyLock::new(|| {
    LocalFilePreprocessor::with_config(PreprocessingConfig {
        max_dimension: MAX_DIMENSION,
        convert_to_webp: true,
        allowed_mime_types: ALLOWED_MIME_TYPES.iter().map(|s| s.to_string()).collect(),
    })
});

/// Validate that the MIME type is in the allowlist.
///
/// Delegates to `file_preprocess::validate_mime_type`.
pub fn validate_mime_type(content_type: &str) -> Result<(), AppError> {
    super::file_preprocess::validate_mime_type(
        content_type,
        &LEGACY_PREPROCESSOR.config.allowed_mime_types,
    )
}

/// Process an uploaded image buffer (resize + WebP conversion).
///
/// Delegates to `LocalFilePreprocessor::preprocess`.
/// Returns `(processed_buffer, "image/webp")`.
pub fn preprocess_profile_image(
    buffer: &[u8],
    content_type: &str,
) -> Result<(Vec<u8>, String), AppError> {
    let result = LEGACY_PREPROCESSOR.preprocess(buffer, content_type, "image")?;
    Ok((result.buffer, result.content_type))
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_mime_type_accepts_jpeg() {
        assert!(validate_mime_type("image/jpeg").is_ok());
    }

    #[test]
    fn validate_mime_type_accepts_png() {
        assert!(validate_mime_type("image/png").is_ok());
    }

    #[test]
    fn validate_mime_type_accepts_webp() {
        assert!(validate_mime_type("image/webp").is_ok());
    }

    #[test]
    fn validate_mime_type_rejects_pdf() {
        assert!(validate_mime_type("application/pdf").is_err());
    }

    #[test]
    fn validate_mime_type_rejects_svg() {
        assert!(validate_mime_type("image/svg+xml").is_err());
    }

    #[test]
    fn validate_mime_type_rejects_empty_string() {
        assert!(validate_mime_type("").is_err());
    }
}
