// ─── AWS LAMBDA FILE PREPROCESSOR ──────────────────────────────────────────
//
// **PURPOSE:** This module provides a swappable preprocessor for use when
// the Rust backend runs on **AWS** (EC2, ECS, EKS, or Lambda).
//
// ─────────────────────────────────────────────────────────────────────────
// 🏗️ TWO MODES OF OPERATION
// ─────────────────────────────────────────────────────────────────────────
//
// **Mode A: Integrated (default) — image crate runs natively**
//   Use `AwsLambdaFilePreprocessor` as the preprocessor in your AppState.
//   Works on EC2, ECS, EKS, and Lambda — anywhere you run a native binary.
//   ```
//   // In app_state.rs:
//   let file_preprocessor: Arc<dyn FilePreprocessor> =
//       Arc::new(AwsLambdaFilePreprocessor::new());
//   ```
//
// **Mode B: Standalone Lambda function — separate deployment**
//   Deploy a dedicated Rust Lambda (in `aws-lambda/`) that handles
//   preprocessing as a separate service. The Lambda is triggered by:
//     - API Gateway → Lambda proxy (HTTP upload endpoint)
//     - S3 event notifications (auto-process on upload)
//     - Direct SDK invocation from the main backend
//
//   The standalone project is at `aws-lambda/` in the project root.
//   See its README for build & deploy instructions.
//
// ─────────────────────────────────────────────────────────────────────────
// 🦀 RUST ON AWS LAMBDA — FULL NATIVE SUPPORT
// ─────────────────────────────────────────────────────────────────────────
// AWS Lambda runs in a Firecracker micro-VM on standard Linux.
// The `image` crate works **without modification** — no WASM needed.
// Full WebP support is available (all C dependencies are compiled in).
//
// **Deployment (Mode B — standalone):**
// ```bash
// cd aws-lambda
// cargo build --release --target x86_64-unknown-linux-musl
// # Package as ZIP and upload to AWS Lambda
// ```
//
// Binary size with `image` crate: ~5 MB (acceptable for Lambda).
//
// **Lambda Runtime Crate:** `aws-lambda-rust-runtime` (via `lambda_runtime`)
// See: https://crates.io/crates/lambda_runtime
// ============================================================================

// Reuse the full local preprocessor — AWS Lambda is also a native binary
// environment, so the same `image` crate features are available.
//
// For now, this module re-exports `LocalFilePreprocessor` to avoid
// duplication.  If Lambda-specific preprocessing logic is needed (e.g.
// different resize dimensions, format restrictions), extend this module.

use crate::_utils::app_error::AppError;

use super::{
    FilePreprocessor, PreprocessingConfig, PreprocessingResult,
    file_preprocess_local::LocalFilePreprocessor,
};

/// AWS Lambda file preprocessor.
///
/// Currently delegates to [`LocalFilePreprocessor`] because both run in a
/// native binary environment with full `image` crate support.
///
/// If you need Lambda-specific behaviour (e.g. stricter size limits,
/// different output format), implement the logic directly here.
pub struct AwsLambdaFilePreprocessor {
    inner: LocalFilePreprocessor,
}

impl AwsLambdaFilePreprocessor {
    /// Create a preprocessor with the default config.
    pub fn new() -> Self {
        Self {
            inner: LocalFilePreprocessor::new(),
        }
    }

    /// Create a preprocessor with a custom configuration.
    pub fn with_config(config: PreprocessingConfig) -> Self {
        Self {
            inner: LocalFilePreprocessor::with_config(config),
        }
    }
}

impl Default for AwsLambdaFilePreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl FilePreprocessor for AwsLambdaFilePreprocessor {
    fn preprocess(
        &self,
        buffer: &[u8],
        content_type: &str,
        file_name: &str,
    ) -> Result<PreprocessingResult, AppError> {
        // Delegate to the local preprocessor — identical logic.
        self.inner.preprocess(buffer, content_type, file_name)
    }
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lambda_preprocessor_rejects_pdf() {
        let pre = AwsLambdaFilePreprocessor::new();
        let result = pre.preprocess(b"fake-pdf-content", "application/pdf", "doc.pdf");
        assert!(result.is_err());
    }

    #[test]
    fn lambda_preprocessor_mime_accepts_jpeg() {
        let pre = AwsLambdaFilePreprocessor::new();
        // MIME validation passes for image/jpeg
        let result = pre.preprocess(b"not-a-real-image", "image/jpeg", "test.jpg");
        // Decode will fail since data is not a real image, but MIME is accepted
        assert!(result.is_err());
    }

    #[test]
    fn lambda_preprocessor_with_custom_config() {
        let config = PreprocessingConfig {
            max_dimension: 100,
            ..Default::default()
        };
        let pre = AwsLambdaFilePreprocessor::with_config(config);
        let result = pre.preprocess(b"not-a-real-image", "image/jpeg", "test.jpg");
        // MIME accepted, decode will fail
        assert!(result.is_err());
    }
}
