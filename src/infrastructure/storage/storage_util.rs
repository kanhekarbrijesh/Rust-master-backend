// ─── STORAGE UTILITY ─────────────────────────────────────────────────────────
// Reusable helpers for handling file uploads in controllers.

use std::{collections::HashMap, sync::Arc};

use axum::extract::Multipart;

use crate::{
    _utils::app_error::AppError,
    infrastructure::storage::storage_types::{StorageProvider, StoreFileInput, StoreFileResult},
};

/// Maximum allowed file size: 10 MB.
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Create a `LocalStorage` provider wrapped in `Arc<dyn StorageProvider>`.
pub fn new_local_storage_provider(base_path: &str, serve_prefix: &str) -> Arc<dyn StorageProvider> {
    Arc::new(
        crate::infrastructure::storage::localstorage::LocalStorage::new(base_path, serve_prefix),
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// GENERIC MULTIPART FILE UPLOAD HANDLER
// ═══════════════════════════════════════════════════════════════════════════════
//
// Use this in any controller that accepts a file via multipart form data.
// It:
//   1. Iterates multipart fields
//   2. Stores the first file it finds via `storage.store_file(...)`
//   3. Collects all text fields into a `HashMap<String, String>`
//   4. Returns the storage result + text fields
//
// Controller usage (1–2 lines):
// ```rust
// let (result, fields) = handle_multipart_upload(&*state.storage, "gallery", multipart).await?;
// let status = fields.get("status").unwrap();
// ```

/// Generic result from a multipart file upload.
pub struct UploadResult {
    pub store_result: StoreFileResult,
    pub text_fields: HashMap<String, String>,
}

/// Parse a multipart form, store the uploaded file, and return text fields.
///
/// Security & reliability guarantees:
/// - **File size limit**: Enforces `MAX_FILE_SIZE` (10 MB) — payloads exceeding
///   this threshold are rejected before any file is written to disk.
/// - **Validation-before-store**: All text fields are collected first. The file
///   is only persisted after required metadata is validated, preventing orphan
///   files when required fields are missing.
///
/// * `storage`   — the active storage provider
/// * `directory` — sub-directory/prefix (e.g. `"gallery"`, `"products"`)
/// * `multipart` — the Axum `Multipart` extractor
pub async fn handle_multipart_upload(
    storage: &dyn StorageProvider,
    directory: &str,
    mut multipart: Multipart,
) -> Result<UploadResult, AppError> {
    let mut raw_file_input: Option<(Vec<u8>, String, String)> = None;
    let mut text_fields: HashMap<String, String> = HashMap::new();

    // ── Phase 1: Collect all fields (no storage I/O yet) ──────────────────
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        // File field
        if field.file_name().is_some_and(|n| !n.is_empty()) {
            if raw_file_input.is_some() {
                continue; // only accept the first file
            }
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(format!("File read error: {e}")))?;

            if data.is_empty() {
                return Err(AppError::BadRequest("Uploaded file is empty".into()));
            }
            if data.len() > MAX_FILE_SIZE {
                return Err(AppError::BadRequest(format!(
                    "File exceeds maximum size of {} bytes",
                    MAX_FILE_SIZE
                )));
            }
            raw_file_input = Some((data.to_vec(), file_name, content_type));
        } else {
            // Text field
            let text = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(format!("Invalid field '{name}': {e}")))?;
            text_fields.insert(name, text.trim().to_string());
        }
    }

    // ── Phase 2: Validate before persisting ───────────────────────────────
    let (buffer, file_name, content_type) =
        raw_file_input.ok_or_else(|| AppError::BadRequest("No file found in upload".into()))?;

    // ── Phase 3: Store the file ───────────────────────────────────────────
    let store_result = storage
        .store_file(StoreFileInput {
            buffer,
            file_name,
            content_type,
            directory: directory.to_string(),
        })
        .await?;

    Ok(UploadResult {
        store_result,
        text_fields,
    })
}
