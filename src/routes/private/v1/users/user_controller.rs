use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    _utils::app_error::AppError,
    domain::users::user_dto::{UserCreateDto, UserUpdateDto},
    infrastructure::{app_state::AppState, storage::storage_util},
    services::domain_services::user_services,
};

pub struct UserController;

impl UserController {
    // ==========================================
    // 1. CREATE (Post) — Multipart file upload
    // ==========================================
    pub async fn create(
        State(state): State<AppState>,
        multipart: Multipart,
    ) -> Result<impl IntoResponse, AppError> {
        // Centralised: parses multipart, validates MIME, preprocesses image
        // (resize + WebP), stores the file — all in one call.
        let upload = storage_util::handle_multipart_upload_with_preprocessing(
            &*state.storage,
            "users",
            multipart,
        )
        .await?;

        let name = upload
            .text_fields
            .get("name")
            .ok_or_else(|| AppError::BadRequest("Missing field: name".into()))?
            .clone();
        let role_id: i32 = upload
            .text_fields
            .get("role_id")
            .ok_or_else(|| AppError::BadRequest("Missing field: role_id".into()))?
            .parse()
            .map_err(|_| {
                AppError::BadRequest("Invalid field: role_id must be an integer".into())
            })?;

        let dto = UserCreateDto {
            name,
            profile_image: upload.store_result.url.clone(),
            role_id,
        };

        // ── Persist to DB · rollback file on failure ──────────────────────
        match user_services::create_user(&state.psql_pool, dto).await {
            Ok(user) => Ok((StatusCode::CREATED, Json(serde_json::json!(user)))),

            Err(db_err) => {
                storage_util::spawn_orphan_cleanup(state.storage.clone(), &upload.store_result.key);
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
        multipart: Multipart,
    ) -> Result<impl IntoResponse, AppError> {
        // Centralised: parses multipart with optional file + preprocessing.
        let upload = storage_util::handle_multipart_upload_optional_with_preprocessing(
            &*state.storage,
            "users",
            multipart,
        )
        .await?;

        let name = upload
            .text_fields
            .get("name")
            .ok_or_else(|| AppError::BadRequest("Missing field: name".into()))?
            .clone();
        let role_id: i32 = upload
            .text_fields
            .get("role_id")
            .ok_or_else(|| AppError::BadRequest("Missing field: role_id".into()))?
            .parse()
            .map_err(|_| {
                AppError::BadRequest("Invalid field: role_id must be an integer".into())
            })?;

        let dto = UserUpdateDto {
            id,
            name,
            profile_image: upload.store_result.as_ref().map(|r| r.url.clone()),
            role_id,
        };

        match user_services::update_user(&state.psql_pool, id, dto).await {
            Ok(user) => Ok((StatusCode::OK, Json(serde_json::json!(user)))),

            Err(db_err) => {
                // Orphan-protection: new file was stored but DB update failed.
                if let Some(ref result) = upload.store_result {
                    storage_util::spawn_orphan_cleanup(state.storage.clone(), &result.key);
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
        // Also deletes the associated profile image from storage.
        user_services::delete_user(
            &state.psql_pool,
            state.storage.clone(),
            &state.storage_serve_prefix,
            id,
        )
        .await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "User deleted successfully" })),
        ))
    }
}
