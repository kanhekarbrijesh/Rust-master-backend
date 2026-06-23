use std::collections::HashMap;

use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    _utils::app_error::AppError,
    domain::users::user_dto::{UserCreateDto, UserUpdateDto},
    infrastructure::{
        app_state::AppState,
        storage::{
            image_preprocessing,
            storage_types::{StoreFileInput, StoreFileResult},
        },
    },
    services::domain_services::user_services,
};

/// Maximum allowed file size: 10 MB.
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

pub struct UserController;

impl UserController {
    // ==========================================
    // 1. CREATE (Post) — Multipart file upload
    // ==========================================
    pub async fn create(
        State(state): State<AppState>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, AppError> {
        let mut text_fields: HashMap<String, String> = HashMap::new();
        let mut store_result: Option<StoreFileResult> = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
        {
            let name = field.name().unwrap_or("").to_string();

            if field.file_name().is_some_and(|n| !n.is_empty()) {
                if store_result.is_some() {
                    continue;
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

                // ── MIME validation ───────────────────────────────────────
                image_preprocessing::validate_mime_type(&content_type)?;

                // ── Preprocess: resize + WebP conversion ──────────────────
                let (processed_buf, webp_content_type) =
                    image_preprocessing::preprocess_profile_image(&data, &content_type)?;

                // ── Store the preprocessed file ───────────────────────────
                let result = state
                    .storage
                    .store_file(StoreFileInput {
                        buffer: processed_buf,
                        file_name: format!(
                            "{}.webp",
                            file_name.rsplit('.').last().unwrap_or(file_name.as_str())
                        ),
                        content_type: webp_content_type,
                        directory: "users".to_string(),
                    })
                    .await?;
                store_result = Some(result);
            } else {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid field '{name}': {e}")))?;
                text_fields.insert(name, text.trim().to_string());
            }
        }

        let store_result = store_result
            .ok_or_else(|| AppError::BadRequest("Missing file: profile_image".into()))?;

        let name = text_fields
            .get("name")
            .ok_or_else(|| AppError::BadRequest("Missing field: name".into()))?
            .clone();
        let role_id_str = text_fields
            .get("role_id")
            .ok_or_else(|| AppError::BadRequest("Missing field: role_id".into()))?
            .clone();
        let role_id: i32 = role_id_str.parse().map_err(|_| {
            AppError::BadRequest("Invalid field: role_id must be an integer".into())
        })?;

        let dto = UserCreateDto {
            name,
            profile_image: store_result.url.clone(),
            role_id,
        };

        // ── Persist to DB · rollback file on failure ──────────────────────
        match user_services::create_user(&state.psql_pool, dto).await {
            Ok(user) => Ok((StatusCode::CREATED, Json(serde_json::json!(user)))),

            Err(db_err) => {
                // Orphan-protection: file was stored but DB insert failed.
                let storage = state.storage.clone();
                let key = store_result.key.clone();
                tokio::spawn(async move {
                    let _ = storage.delete_file(&key).await;
                });
                Err(db_err)
            }
        }
    }

    // ==========================================
    // 2. READ ALL (Get)
    // ==========================================
    pub async fn read_all(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
        let users = user_services::get_all_users(&state.psql_pool).await?;
        Ok((StatusCode::OK, Json(users)))
    }

    // ==========================================
    // 3. READ BY ID (Get)
    // ==========================================
    pub async fn read_by_id(
        State(state): State<AppState>,
        Path(id): Path<i32>,
    ) -> Result<impl IntoResponse, AppError> {
        let user = user_services::get_user_by_id(&state.psql_pool, id).await?;
        Ok((StatusCode::OK, Json(user)))
    }

    // ==========================================
    // 4. UPDATE (Put) — Multipart (file is optional)
    // ==========================================
    pub async fn update(
        State(state): State<AppState>,
        Path(id): Path<i32>,
        mut multipart: Multipart,
    ) -> Result<impl IntoResponse, AppError> {
        let mut text_fields: HashMap<String, String> = HashMap::new();
        let mut file_result: Option<StoreFileResult> = None;

        // Parse multipart fields manually — file is optional for update
        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart error: {e}")))?
        {
            let name = field.name().unwrap_or("").to_string();

            if field.file_name().is_some_and(|n| !n.is_empty()) {
                if file_result.is_some() {
                    continue;
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

                // ── MIME validation ───────────────────────────────────────
                image_preprocessing::validate_mime_type(&content_type)?;

                // ── Preprocess: resize + WebP conversion ──────────────────
                let (processed_buf, webp_content_type) =
                    image_preprocessing::preprocess_profile_image(&data, &content_type)?;

                let result = state
                    .storage
                    .store_file(StoreFileInput {
                        buffer: processed_buf,
                        file_name: format!(
                            "{}.webp",
                            file_name.rsplit('.').last().unwrap_or(file_name.as_str())
                        ),
                        content_type: webp_content_type,
                        directory: "users".to_string(),
                    })
                    .await?;
                file_result = Some(result);
            } else {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("Invalid field '{name}': {e}")))?;
                text_fields.insert(name, text.trim().to_string());
            }
        }

        let name = text_fields
            .get("name")
            .ok_or_else(|| AppError::BadRequest("Missing field: name".into()))?
            .clone();
        let role_id_str = text_fields
            .get("role_id")
            .ok_or_else(|| AppError::BadRequest("Missing field: role_id".into()))?
            .clone();
        let role_id: i32 = role_id_str.parse().map_err(|_| {
            AppError::BadRequest("Invalid field: role_id must be an integer".into())
        })?;

        let dto = UserUpdateDto {
            id,
            name,
            profile_image: file_result.as_ref().map(|r| r.url.clone()),
            role_id,
        };

        match user_services::update_user(&state.psql_pool, id, dto).await {
            Ok(user) => Ok((StatusCode::OK, Json(serde_json::json!(user)))),

            Err(db_err) => {
                // Orphan-protection: new file was stored but DB update failed.
                if let Some(result) = file_result {
                    let storage = state.storage.clone();
                    let key = result.key.clone();
                    tokio::spawn(async move {
                        let _ = storage.delete_file(&key).await;
                    });
                }
                Err(db_err)
            }
        }
    }

    // ==========================================
    // 5. DELETE (Delete)
    // ==========================================
    pub async fn delete(
        State(state): State<AppState>,
        Path(id): Path<i32>,
    ) -> Result<impl IntoResponse, AppError> {
        user_services::delete_user(&state.psql_pool, id).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "User deleted successfully" })),
        ))
    }
}
