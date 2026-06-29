// ─── CLOUDFLARE WORKER FILE PREPROCESSOR ───────────────────────────────────
//
// **PURPOSE:** This module provides a validation-only preprocessor for use
// when the Rust backend is deployed behind a Cloudflare Worker. The Worker
// itself can be written in **Rust** (via `workers-rs`), **JavaScript**, or
// **TypeScript**.
//
// ─────────────────────────────────────────────────────────────────────────
// 🔥 CAN WE USE RUST FOR THE CLOUDFLARE WORKER? YES.
// ─────────────────────────────────────────────────────────────────────────
//
// **Option A: `workers-rs` (Rust → WASM) — RECOMMENDED**
//   Use the `worker` crate to write the entire Worker in Rust.
//   It compiles to WASM and runs in the V8 isolate.
//   ```
//   npx wrangler init cloudflare-worker --wasm
//   cd cloudflare-worker
//   cargo generate --git https://github.com/cloudflare/workers-rs
//   ```
//   The `image` crate can be compiled to WASM (see note below).
//
// **Option B: JS/TS Worker calling Rust WASM**
//   Write the Worker shell in JS/TS, compile Rust preprocessing logic to
//   WASM via `wasm-pack`, and import the WASM module in the Worker.
//
// **Option C: JS/TS Worker (proxy only)**
//   A simple JS/TS Worker that validates + proxies to backend. No WASM.
//
// ─────────────────────────────────────────────────────────────────────────
// 🦀 `workers-rs` — Rust Cloudflare Worker Library
// ─────────────────────────────────────────────────────────────────────────
// GitHub: https://github.com/cloudflare/workers-rs
// Crate:  https://crates.io/crates/worker
// Guide:  https://github.com/cloudflare/workers-rs#quick-start
//
// The `worker` crate gives you:
//   - `#[event(fetch)]` — HTTP request handler
//   - `worker::Bucket` — R2 bucket binding
//   - `worker::Request` / `worker::Response` — HTTP primitives
//   - Full access to Cloudflare APIs (KV, D1, R2, Queues, etc.)
//
// ─────────────────────────────────────────────────────────────────────────
// 📦 Image Processing in WASM
// ─────────────────────────────────────────────────────────────────────────
// The `image` crate can be compiled to WASM (it has WASM support).
// However, the `webp` feature may not work in WASM environments. Alternatives:
//   1. Use `tiny_skia` + `image` (no WebP) for resize-only operations
//   2. Use the `image` crate with `jpeg`/`png` features (no WebP)
//   3. Use `photon-rs` — a Rust image library designed for WASM
//      (https://github.com/silvia-odwyer/photon)
//
// See `cloudflare-worker/` at the project root for the actual Worker project.
// ============================================================================

// NOTE: This module is used by the main Rust API when running behind a
// Cloudflare Worker. It validates MIME + file size and passes through.
// The actual image preprocessing (if needed) happens in the Worker itself
// (using the Rust `worker` crate compiled to WASM) or on the client side.

use crate::_utils::app_error::AppError;

use super::{FilePreprocessor, PreprocessingConfig, PreprocessingResult, validate_mime_type};

/// Maximum file size for Cloudflare Worker uploads (10 MB).
const CF_MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Cloudflare Worker file preprocessor.
///
/// Unlike the local preprocessor, this does **not** do image transformation.
/// It validates the file and passes it through as-is.
///
/// **Why?** Because Cloudflare Workers run in a V8 isolate where native
/// Rust crates (like `image`) cannot be used directly without WASM
/// compilation.
///
/// **Enhancement path:**
///   - Compile `image` crate to WASM via `wasm-pack`
///   - Add WASM image processing in this file behind a `cfg(target_arch = "wasm32")` gate
///   - Deploy the WASM binary as a Cloudflare Worker
pub struct CfWorkerFilePreprocessor {
    config: PreprocessingConfig,
}

impl CfWorkerFilePreprocessor {
    /// Create a preprocessor with the default config.
    pub fn new() -> Self {
        Self {
            config: PreprocessingConfig::default(),
        }
    }

    /// Create a preprocessor with a custom configuration.
    pub fn with_config(config: PreprocessingConfig) -> Self {
        Self { config }
    }

    /// Check file size against the maximum allowed.
    fn check_file_size(buffer: &[u8]) -> Result<(), AppError> {
        if buffer.len() > CF_MAX_FILE_SIZE {
            return Err(AppError::BadRequest(format!(
                "File exceeds maximum size of {} bytes",
                CF_MAX_FILE_SIZE
            )));
        }
        if buffer.is_empty() {
            return Err(AppError::BadRequest("Uploaded file is empty".into()));
        }
        Ok(())
    }
}

impl Default for CfWorkerFilePreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePreprocessor for CfWorkerFilePreprocessor {
    fn preprocess(
        &self,
        buffer: &[u8],
        content_type: &str,
        file_name: &str,
    ) -> Result<PreprocessingResult, AppError> {
        // ── 1. File size validation ──────────────────────────────────────
        Self::check_file_size(buffer)?;

        // ── 2. MIME validation ───────────────────────────────────────────
        validate_mime_type(content_type, &self.config.allowed_mime_types)?;

        // ── 3. Pass through (no transformation) ──────────────────────────
        // In a real Worker with WASM image processing, you would:
        //   - Decode the image buffer
        //   - Resize if exceeding max_dimension
        //   - Re-encode to WebP
        //   - Return the new buffer
        Ok(PreprocessingResult {
            buffer: buffer.to_vec(),
            content_type: content_type.to_string(),
            file_name: file_name.to_string(),
        })
    }
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cf_preprocessor_rejects_pdf() {
        let pre = CfWorkerFilePreprocessor::new();
        let result = pre.preprocess(b"fake-pdf", "application/pdf", "doc.pdf");
        assert!(result.is_err());
    }

    #[test]
    fn cf_preprocessor_accepts_jpeg() {
        let pre = CfWorkerFilePreprocessor::new();
        let result = pre.preprocess(b"fake-jpeg-bytes", "image/jpeg", "photo.jpg");
        assert!(result.is_ok());
        let processed = result.unwrap();
        assert_eq!(processed.content_type, "image/jpeg");
        assert_eq!(processed.file_name, "photo.jpg");
        // Pass-through: buffer unchanged
        assert_eq!(processed.buffer, b"fake-jpeg-bytes");
    }

    #[test]
    fn cf_preprocessor_rejects_empty_file() {
        let pre = CfWorkerFilePreprocessor::new();
        let result = pre.preprocess(b"", "image/jpeg", "empty.jpg");
        assert!(result.is_err());
    }

    #[test]
    fn cf_preprocessor_allows_custom_config() {
        let config = PreprocessingConfig {
            allowed_mime_types: vec!["image/png".into()],
            ..Default::default()
        };
        let pre = CfWorkerFilePreprocessor::with_config(config);
        // PNG is in the list, should pass
        assert!(pre.preprocess(b"fake-png", "image/png", "img.png").is_ok());
        // JPEG is NOT in the list, should fail
        assert!(
            pre.preprocess(b"fake-jpeg", "image/jpeg", "img.jpg")
                .is_err()
        );
    }
}
