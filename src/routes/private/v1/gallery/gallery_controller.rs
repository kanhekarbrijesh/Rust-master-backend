use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};

use crate::{
    _utils::{app_error::AppError, validate_json::ValidatedJson},
    domain::gallery::gallery_dto::{GalleryCreateDto, GalleryDto, GalleryUpdateDto},
    infrastructure::app_state::AppState,
    services::domain_services::gallerly_services,
};
use mongodb::bson::oid::ObjectId;

pub struct GalleryController;

impl GalleryController {
    // ==========================================
    // 1. CREATE (Post)
    // ==========================================
    pub async fn create(
        State(state): State<AppState>,
        ValidatedJson(payload): ValidatedJson<GalleryCreateDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let id = gallerly_services::create_gallery(&state, payload).await?;
        Ok((StatusCode::CREATED, Json(serde_json::json!({"id": id}))))
    }

    // ==========================================
    // 2. READ ALL (Get)
    // ==========================================
    pub async fn read_all(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
        let items = gallerly_services::get_all_gallerys(&state).await?;
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

        let item = gallerly_services::get_gallery_by_id(&state, &obj_id.to_hex()).await?;
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

        gallerly_services::update_gallery(&state, &obj_id.to_hex(), payload).await?;
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

        gallerly_services::delete_gallery(&state, &obj_id.to_hex()).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Gallery item deleted successfully" })),
        ))
    }
}
