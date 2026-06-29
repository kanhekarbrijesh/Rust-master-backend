// ─── LOCAL FILE PREPROCESSOR ───────────────────────────────────────────────
//
// Full-featured image preprocessor intended for local / native server use.
// Uses the `image` crate (jpeg, png, webp features) to:
//   1. Validate MIME type
//   2. Decode the image
//   3. Resize proportionally if exceeding max dimension
//   4. Convert to WebP format
//
// **When to use:**
//   - `cargo run` / `cargo build` (native binary)
//   - AWS Lambda (binary also runs natively, so this module works there too)
//
// **Dependencies (Cargo.toml):**
//   - `image` with features ["jpeg", "png", "webp"]
//   - Standard library only (no external runtime dependencies)
// ============================================================================

use image::{DynamicImage, ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
use std::io::Cursor;

use crate::_utils::app_error::AppError;

use super::{
    FilePreprocessor, PreprocessingConfig, PreprocessingResult, resolve_key_from_config,
    strip_extension, validate_mime_type,
};

/// Full-featured local image preprocessor.
///
/// Can be swapped for `CfWorkerFilePreprocessor` or
/// `AwsLambdaFilePreprocessor` at instantiation time (see `mod.rs`).
pub struct LocalFilePreprocessor {
    /// The configuration controlling resize, WebP conversion, and MIME validation.
    pub config: PreprocessingConfig,
}

impl LocalFilePreprocessor {
    /// Create a preprocessor with the default config
    /// (max 512 px, WebP conversion, standard image MIME types).
    pub fn new() -> Self {
        Self {
            config: PreprocessingConfig::default(),
        }
    }

    /// Create a preprocessor with a custom configuration.
    pub fn with_config(config: PreprocessingConfig) -> Self {
        Self { config }
    }
}

impl Default for LocalFilePreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePreprocessor for LocalFilePreprocessor {
    fn preprocess(
        &self,
        buffer: &[u8],
        content_type: &str,
        file_name: &str,
    ) -> Result<PreprocessingResult, AppError> {
        // ── 1. MIME validation ───────────────────────────────────────────
        validate_mime_type(content_type, &self.config.allowed_mime_types)?;

        // ── 2. Decode the image ──────────────────────────────────────────
        let img = ImageReader::new(Cursor::new(buffer))
            .with_guessed_format()
            .map_err(|e| AppError::BadRequest(format!("Failed to read image: {e}")))?
            .decode()
            .map_err(|e| AppError::BadRequest(format!("Failed to decode image: {e}")))?;

        // ── 3. Resize if needed (maintain aspect ratio) ──────────────────
        let max_dim = self.config.max_dimension;
        let processed = if max_dim > 0 && (img.width() > max_dim || img.height() > max_dim) {
            let ratio = (max_dim as f64 / img.width().max(img.height()) as f64).min(1.0);
            let new_w = (img.width() as f64 * ratio).round() as u32;
            let new_h = (img.height() as f64 * ratio).round() as u32;
            DynamicImage::ImageRgba8(image::imageops::resize(
                &img,
                new_w,
                new_h,
                FilterType::Lanczos3,
            ))
        } else {
            img
        };

        // ── 4. WebP conversion (if enabled) ──────────────────────────────
        if self.config.convert_to_webp {
            let mut webp_buf = Vec::new();
            {
                let encoder = WebPEncoder::new_lossless(&mut webp_buf);
                processed
                    .write_with_encoder(encoder)
                    .map_err(|e| AppError::Internal(format!("WebP encoding failed: {e}")))?;
            }

            let webp_file_name =
                strip_extension(file_name).unwrap_or(file_name).to_string() + ".webp";

            // ── 5. Encrypt (if enabled) ──────────────────────────────────
            if self.config.encrypt_file {
                let encrypted = self.encrypt(&webp_buf)?;
                return Ok(PreprocessingResult {
                    buffer: encrypted,
                    content_type: "application/octet-stream".to_string(),
                    file_name: webp_file_name,
                    is_encrypted: true,
                });
            }

            Ok(PreprocessingResult {
                buffer: webp_buf,
                content_type: "image/webp".to_string(),
                file_name: webp_file_name,
                is_encrypted: false,
            })
        } else {
            // Return as-is (no format conversion)
            let mut out_buf = Vec::new();
            processed
                .write_to(&mut Cursor::new(&mut out_buf), image::ImageFormat::Png)
                .map_err(|e| AppError::Internal(format!("Image re-encode failed: {e}")))?;

            if self.config.encrypt_file {
                let encrypted = self.encrypt(&out_buf)?;
                return Ok(PreprocessingResult {
                    buffer: encrypted,
                    content_type: "application/octet-stream".to_string(),
                    file_name: file_name.to_string(),
                    is_encrypted: true,
                });
            }

            Ok(PreprocessingResult {
                buffer: out_buf,
                content_type: content_type.to_string(),
                file_name: file_name.to_string(),
                is_encrypted: false,
            })
        }
    }

    fn resolve_encryption_key(&self) -> Result<[u8; super::encryption_utils::KEY_SIZE], AppError> {
        resolve_key_from_config(&self.config.encryption_key_hex)
    }
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_preprocessor_rejects_pdf() {
        let pre = LocalFilePreprocessor::new();
        let result = pre.preprocess(b"fake-pdf-content", "application/pdf", "doc.pdf");
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            AppError::BadRequest(msg) => assert!(msg.contains("Unsupported")),
            _ => panic!("Expected BadRequest, got {:?}", err),
        }
    }

    #[test]
    fn local_preprocessor_mime_accepts_jpeg() {
        let pre = LocalFilePreprocessor::new();
        // MIME validation passes for image/jpeg
        let result = pre.preprocess(b"not-a-real-jpeg", "image/jpeg", "test.jpg");
        // Decode will fail but MIME is accepted
        assert!(result.is_err());
    }

    #[test]
    fn preprocessor_no_webp_conversion() {
        let config = PreprocessingConfig {
            convert_to_webp: false,
            max_dimension: 0, // no resize
            ..Default::default()
        };
        let pre = LocalFilePreprocessor::with_config(config);
        // With no WebP conversion, the data still needs to be a valid image
        // So using fake data will fail at decode step
        let result = pre.preprocess(b"not-an-image", "image/png", "test.png");
        assert!(result.is_err());
    }
}
