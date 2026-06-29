// ─── CENTRALIZED FILE HANDLING UTILITY ────────────────────────────────────────
//
// Reusable helpers for every controller that accepts file uploads.
//
// **Why centralise?**
// Every file-handling controller (users, gallery, products, etc.) needs the
// same three things: (1) parse multipart, (2) optionally preprocess images,
// (3) store the file.  On DB failure the already-stored file must be cleaned
// up to prevent orphans.  These helpers eliminate the duplicate code that
// existed across controllers.
//
// **Available helpers** (use the one that fits your flow):
//
// | Helper                                              | File required? | Preprocessing? |
// |-----------------------------------------------------|----------------|----------------|
// | `handle_multipart_upload`                           | ✅ Yes         | ❌ No          |
// | `handle_multipart_upload_with_preprocessing`        | ✅ Yes         | ✅ Yes (image) |
// | `handle_multipart_upload_optional`                  | ❌ No          | ❌ No          |
// | `handle_multipart_upload_optional_with_preprocessing`| ❌ No          | ✅ Yes (image) |
//
// Orphan protection — call `spawn_orphan_cleanup` after a failed DB write:
// ```rust
// Err(db_err) => {
//     spawn_orphan_cleanup(state.storage.clone(), &upload.store_result.key);
//     Err(db_err)
// }
// ```
//
// Old-file cleanup on update — call `resolve_file_url_on_update`:
// ```rust
// let new_url = resolve_file_url_on_update(
//     &state.storage, &old_file_url, new_store_result.as_ref(),
// ).await;
// ```
// ============================================================================

use std::{collections::HashMap, sync::Arc};

use axum::extract::Multipart;

use crate::{
    _utils::app_error::AppError,
    infrastructure::storage::{
        file_preprocess::FilePreprocessor,
        image_preprocessing,
        storage_types::{StorageProvider, StoreFileInput, StoreFileResult},
    },
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
// SHARED TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Generic result from a multipart file upload.
pub struct UploadResult {
    pub store_result: StoreFileResult,
    pub text_fields: HashMap<String, String>,
}

/// Result for **optional** file upload (used in update flows where file is
/// not required).  When no file was provided `store_result` is `None`.
pub struct OptionalUploadResult {
    pub store_result: Option<StoreFileResult>,
    pub text_fields: HashMap<String, String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORPHAN PROTECTION & CLEANUP HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Fire-and-forget deletion — use when a DB write fails *after* the file was
/// already stored, so the orphan file is cleaned up asynchronously.
///
/// # Example
/// ```rust,ignore
/// match db_create(...).await {
///     Ok(id) => Ok(…),
///     Err(db_err) => {
///         spawn_orphan_cleanup(state.storage.clone(), &upload.store_result.key);
///         Err(db_err)
///     }
/// }
/// ```
pub fn spawn_orphan_cleanup(storage: Arc<dyn StorageProvider>, file_key: &str) {
    let key = file_key.to_string();
    tokio::spawn(async move {
        let _ = storage.delete_file(&key).await;
    });
}

/// Delete a file by key, swallowing any error (best-effort).
/// Useful for cleanup in update flows where deleting the old file is
/// desirable but not critical for the request to succeed.
pub async fn delete_file_quietly(storage: &dyn StorageProvider, key: &str) {
    let _ = storage.delete_file(key).await;
}

/// Extract a storage key from a full storage URL.
///
/// The storage URL has the format `{serve_prefix}/{storage_key}`.
/// This helper strips the `{serve_prefix}/` prefix to recover the key
/// that can be passed to `StorageProvider::delete_file()`.
///
/// If the URL does not start with the expected serve prefix, the whole
/// URL is returned as-is (best-effort fallback).
///
/// # Example
/// ```
/// let key = storage_key_from_url("/uploads/users/uuid-image.webp", "/uploads");
/// assert_eq!(key, "users/uuid-image.webp");
/// ```
pub fn storage_key_from_url(url: &str, serve_prefix: &str) -> String {
    let prefix = serve_prefix.trim_end_matches('/');
    url.strip_prefix(prefix)
        .and_then(|s| s.strip_prefix('/'))
        .unwrap_or(url)
        .to_string()
}

/// Given an old file URL (from the DB) and an optional new store result,
/// return the URL that should be persisted.  If a new file was uploaded,
/// the old file is deleted (best-effort).
///
/// **When the user uploads a new file during an update:**
/// 1. The new file is already stored (by the caller).
/// 2. The old file (on disk/S3/R2) is deleted.
/// 3. The new URL is returned so the caller can persist it.
///
/// **When the user does NOT upload a new file:** the old URL is returned
/// unchanged.
pub async fn resolve_file_url_on_update(
    storage: &dyn StorageProvider,
    serve_prefix: &str,
    old_file_url: &str,
    new_store_result: Option<&StoreFileResult>,
) -> String {
    match new_store_result {
        Some(result) => {
            // New file uploaded — delete the old one (best-effort).
            let old_key = storage_key_from_url(old_file_url, serve_prefix);
            delete_file_quietly(storage, &old_key).await;
            result.url.clone()
        }
        None => old_file_url.to_string(),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LOW-LEVEL: COLLECT MULTIPART FIELDS (used internally)
// ═══════════════════════════════════════════════════════════════════════════════

/// Iterate a multipart stream and return (raw_file, text_fields).
/// `accept_file` controls whether a file is required.
async fn collect_multipart_fields(
    mut multipart: Multipart,
    accept_file: bool,
) -> Result<(Option<(Vec<u8>, String, String)>, HashMap<String, String>), AppError> {
    let mut raw_file_input: Option<(Vec<u8>, String, String)> = None;
    let mut text_fields: HashMap<String, String> = HashMap::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();

        // Check if this is a file field by inspecting file_name without consuming
        if field.file_name().is_some_and(|n| !n.is_empty()) {
            let file_name = field.file_name().unwrap_or("unnamed").to_string();
            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();
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

            if raw_file_input.is_none() {
                raw_file_input = Some((data.to_vec(), file_name, content_type));
            }
            // Ignore subsequent file fields (only accept the first)
        } else {
            let text = field
                .text()
                .await
                .map_err(|e| AppError::BadRequest(format!("Invalid field '{name}': {e}")))?;
            text_fields.insert(name, text.trim().to_string());
        }
    }

    if accept_file && raw_file_input.is_none() {
        return Err(AppError::BadRequest("No file found in upload".into()));
    }

    Ok((raw_file_input, text_fields))
}

/// Strip the last file extension from a filename, returning the base name.
/// E.g. `"photo.jpeg"` → `"photo"`, `"photo"` → `None`.
fn strip_extension(file_name: &str) -> Option<&str> {
    let dot = file_name.rfind('.')?;
    if dot == 0 {
        None
    } else {
        Some(&file_name[..dot])
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HIGH-LEVEL HELPERS — use these in your controllers
// ═══════════════════════════════════════════════════════════════════════════════

/// ──────────────────────────────────────────────────────────────────────────────
/// **File required · No preprocessing**
///
/// Parse multipart, store the first file, return the result + text fields.
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload(
    storage: &dyn StorageProvider,
    directory: &str,
    multipart: Multipart,
) -> Result<UploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, true).await?;
    let (buffer, file_name, content_type) = raw_file.unwrap(); // safe: accept_file=true

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

/// ──────────────────────────────────────────────────────────────────────────────
/// **File required · With image preprocessing**
///
/// Same as `handle_multipart_upload` but runs the image through the
/// preprocessing pipeline (MIME validation, resize, WebP conversion) before
/// storing.
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload_with_preprocessing(
    storage: &dyn StorageProvider,
    directory: &str,
    multipart: Multipart,
) -> Result<UploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, true).await?;
    let (buffer, file_name, content_type) = raw_file.unwrap(); // safe: accept_file=true

    // ── MIME validation ───────────────────────────────────────────────────
    image_preprocessing::validate_mime_type(&content_type)?;

    // ── Preprocess: resize + WebP conversion ──────────────────────────────
    let (processed_buf, webp_content_type) =
        image_preprocessing::preprocess_profile_image(&buffer, &content_type)?;

    // ── Store the preprocessed file (always .webp) ────────────────────────
    let webp_file_name = strip_extension(&file_name)
        .unwrap_or(&file_name)
        .to_string()
        + ".webp";
    let store_result = storage
        .store_file(StoreFileInput {
            buffer: processed_buf,
            file_name: webp_file_name,
            content_type: webp_content_type,
            directory: directory.to_string(),
        })
        .await?;

    Ok(UploadResult {
        store_result,
        text_fields,
    })
}

/// ──────────────────────────────────────────────────────────────────────────────
/// **File optional · No preprocessing**
///
/// For update endpoints where the file is optional.  Returns
/// `OptionalUploadResult` — when no file was provided `store_result` is `None`.
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload_optional(
    storage: &dyn StorageProvider,
    directory: &str,
    multipart: Multipart,
) -> Result<OptionalUploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, false).await?;

    let store_result = match raw_file {
        Some((buffer, file_name, content_type)) => Some(
            storage
                .store_file(StoreFileInput {
                    buffer,
                    file_name,
                    content_type,
                    directory: directory.to_string(),
                })
                .await?,
        ),
        None => None,
    };

    Ok(OptionalUploadResult {
        store_result,
        text_fields,
    })
}

/// ──────────────────────────────────────────────────────────────────────────────
/// **File optional · With image preprocessing**
///
/// For update endpoints where the file is optional but when present it should
/// be preprocessed (MIME validation, resize, WebP).
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload_optional_with_preprocessing(
    storage: &dyn StorageProvider,
    directory: &str,
    multipart: Multipart,
) -> Result<OptionalUploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, false).await?;

    let store_result = match raw_file {
        Some((buffer, file_name, content_type)) => {
            // ── MIME validation ───────────────────────────────────────────
            image_preprocessing::validate_mime_type(&content_type)?;

            // ── Preprocess: resize + WebP conversion ──────────────────────
            let (processed_buf, webp_content_type) =
                image_preprocessing::preprocess_profile_image(&buffer, &content_type)?;

            let webp_file_name = strip_extension(&file_name)
                .unwrap_or(&file_name)
                .to_string()
                + ".webp";
            Some(
                storage
                    .store_file(StoreFileInput {
                        buffer: processed_buf,
                        file_name: webp_file_name,
                        content_type: webp_content_type,
                        directory: directory.to_string(),
                    })
                    .await?,
            )
        }
        None => None,
    };

    Ok(OptionalUploadResult {
        store_result,
        text_fields,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT-BASED PREPROCESSING HELPERS (swappable at code level)
// ═══════════════════════════════════════════════════════════════════════════════
//
// These functions accept a `&dyn FilePreprocessor` instead of calling the
// legacy `image_preprocessing` module directly.
//
// **Usage:**
// ```rust
// use crate::infrastructure::storage::{
//     file_preprocess::file_preprocess_local::LocalFilePreprocessor,
//     storage_util::handle_multipart_upload_with_preprocessor,
// };
//
// let preprocessor = LocalFilePreprocessor::new();
// let result = handle_multipart_upload_with_preprocessor(
//     &*state.storage, &preprocessor, "gallery", multipart,
// ).await?;
// ```
// ============================================================================

/// ──────────────────────────────────────────────────────────────────────────────
/// **File required · With trait-based FilePreprocessor**
///
/// Same as `handle_multipart_upload_with_preprocessing` but accepts a
/// `&dyn FilePreprocessor` so the preprocessing logic can be swapped at
/// compile time.
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload_with_preprocessor(
    storage: &dyn StorageProvider,
    preprocessor: &dyn FilePreprocessor,
    directory: &str,
    multipart: Multipart,
) -> Result<UploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, true).await?;
    let (buffer, file_name, content_type) = raw_file.unwrap(); // safe: accept_file=true

    // ── Preprocess via the injected preprocessor ─────────────────────────
    let result = preprocessor.preprocess(&buffer, &content_type, &file_name)?;

    // ── Store the preprocessed file ──────────────────────────────────────
    let store_result = storage
        .store_file(StoreFileInput {
            buffer: result.buffer,
            file_name: result.file_name,
            content_type: result.content_type,
            directory: directory.to_string(),
        })
        .await?;

    Ok(UploadResult {
        store_result,
        text_fields,
    })
}

/// ──────────────────────────────────────────────────────────────────────────────
/// **File optional · With trait-based FilePreprocessor**
///
/// For update endpoints where the file is optional but when present it should
/// be preprocessed via a swappable `FilePreprocessor`.
/// ──────────────────────────────────────────────────────────────────────────────
pub async fn handle_multipart_upload_optional_with_preprocessor(
    storage: &dyn StorageProvider,
    preprocessor: &dyn FilePreprocessor,
    directory: &str,
    multipart: Multipart,
) -> Result<OptionalUploadResult, AppError> {
    let (raw_file, text_fields) = collect_multipart_fields(multipart, false).await?;

    let store_result = match raw_file {
        Some((buffer, file_name, content_type)) => {
            // ── Preprocess via the injected preprocessor ─────────────────
            let result = preprocessor.preprocess(&buffer, &content_type, &file_name)?;

            Some(
                storage
                    .store_file(StoreFileInput {
                        buffer: result.buffer,
                        file_name: result.file_name,
                        content_type: result.content_type,
                        directory: directory.to_string(),
                    })
                    .await?,
            )
        }
        None => None,
    };

    Ok(OptionalUploadResult {
        store_result,
        text_fields,
    })
}
