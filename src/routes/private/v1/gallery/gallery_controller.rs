use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    _utils::{app_error::AppError, validate_json::ValidatedJson},
    domain::gallery::gallery_dto::{GalleryCreateDto, GalleryDto, GalleryUpdateDto},
    infrastructure::{app_state::AppState, storage::storage_util},
    services::domain_services::gallery_services,
};
use mongodb::bson::oid::ObjectId;

pub struct GalleryController;

impl GalleryController {
    // ==========================================
    // 1. CREATE (Post) — Multipart file upload
    // ==========================================
    pub async fn create(
        State(state): State<AppState>,
        multipart: Multipart,
    ) -> Result<impl IntoResponse, AppError> {
        // Centralised: parses multipart, stores the file, returns result.
        let upload =
            storage_util::handle_multipart_upload(&*state.storage, "gallery", multipart).await?;
        let status = upload
            .text_fields
            .get("status")
            .ok_or_else(|| AppError::BadRequest("Missing field: status".into()))?
            .clone();

        let dto = GalleryCreateDto {
            url: upload.store_result.url.clone(),
            status,
        };

        // ── Persist to DB · rollback file on failure ──────────────────────
        match gallery_services::create_gallery(&state, dto).await {
            Ok(id) => Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id})))),

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
        let items = gallery_services::get_all_galleries(&state).await?;
        let dtos: Vec<GalleryDto> = items.into_iter().map(Into::into).collect();
        Ok((StatusCode::OK, Json(dtos)))
    }

    // ==========================================
    // 3. READ BY ID (Get)
    // ==========================================
    pub async fn read_by_id(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        let item = gallery_services::get_gallery_by_id(&state, obj_id).await?;
        let dto: GalleryDto = item.into();
        Ok((StatusCode::OK, Json(dto)))
    }

    // ==========================================
    // 4. UPDATE (Put)
    // ==========================================
    pub async fn update(
        State(state): State<AppState>,
        Path(id): Path<String>,
        ValidatedJson(payload): ValidatedJson<GalleryUpdateDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        gallery_services::update_gallery(&state, obj_id, payload).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Gallery item updated successfully" })),
        ))
    }

    // ==========================================
    // 5. DELETE (Delete)
    // ==========================================
    pub async fn delete(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        // Also deletes the associated file from storage.
        gallery_services::delete_gallery(&state, &state.storage_serve_prefix, obj_id).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Gallery item deleted successfully" })),
        ))
    }
}
