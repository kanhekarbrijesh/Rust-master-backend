// ─── IMAGE PREPROCESSING PIPELINE ─────────────────────────────────────────────
// Edge preprocessing for uploaded images: resize + convert to WebP.
// This runs before the file is stored, reducing storage costs and bandwidth.

use image::{DynamicImage, ImageReader, codecs::webp::WebPEncoder, imageops::FilterType};
use std::io::Cursor;

use crate::_utils::app_error::AppError;

/// Maximum dimension (width or height) for profile images.
/// Images larger than this will be resized proportionally.
const MAX_DIMENSION: u32 = 512;

/// Allowed MIME types for image uploads.
pub const ALLOWED_MIME_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp", "image/gif"];

/// Validate that the MIME type is in the allowlist.
pub fn validate_mime_type(content_type: &str) -> Result<(), AppError> {
    if ALLOWED_MIME_TYPES.contains(&content_type) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
            "Unsupported file type '{}'. Allowed: {}",
            content_type,
            ALLOWED_MIME_TYPES.join(", ")
        )))
    }
}

/// Process an uploaded image buffer:
/// 1. Decode the image
/// 2. Resize proportionally if exceeding MAX_DIMENSION
/// 3. Convert to WebP format
///
/// Returns (processed_buffer, "image/webp" as new content type).
pub fn preprocess_profile_image(
    buffer: &[u8],
    _original_content_type: &str,
) -> Result<(Vec<u8>, String), AppError> {
    // Decode the image
    let img = ImageReader::new(Cursor::new(buffer))
        .with_guessed_format()
        .map_err(|e| AppError::BadRequest(format!("Failed to read image: {e}")))?
        .decode()
        .map_err(|e| AppError::BadRequest(format!("Failed to decode image: {e}")))?;

    // Resize if needed (maintain aspect ratio)
    let processed = if img.width() > MAX_DIMENSION || img.height() > MAX_DIMENSION {
        let ratio = (MAX_DIMENSION as f64 / img.width().max(img.height()) as f64).min(1.0);
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

    // Encode as WebP
    let mut webp_buf = Vec::new();
    {
        let encoder = WebPEncoder::new_lossless(&mut webp_buf);
        processed
            .write_with_encoder(encoder)
            .map_err(|e| AppError::Internal(format!("WebP encoding failed: {e}")))?;
    }

    Ok((webp_buf, "image/webp".to_string()))
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
