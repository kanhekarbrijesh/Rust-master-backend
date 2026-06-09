// src/routes/v1/products/product_controller.rs
use crate::{
    _utils::{app_error::AppError, validate_json::ValidatedJson},
    domain::products::product_dto::ProductDto,
    infrastructure::app_state::AppState,
    services::domain_services::product_services,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use mongodb::bson::oid::ObjectId;

pub struct ProductController;

impl ProductController {
    // ==========================================
    // 1. CREATE (Post)
    // ==========================================
    pub async fn create(
        State(state): State<AppState>,
        ValidatedJson(payload): ValidatedJson<ProductDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let generated_id = product_services::create_product(&state, payload).await?;
        Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({ "id": generated_id })),
        ))
    }

    // ==========================================
    // 2. READ ALL (Get)
    // ==========================================
    pub async fn read_all(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
        let products = product_services::get_all_products(&state).await?;
        Ok((StatusCode::OK, Json(products)))
    }

    // ==========================================
    // 3. READ BY ID (Get)
    // ==========================================
    pub async fn read_by_id(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        // Sanitize incoming path parameter formats
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        let product = product_services::get_product_by_id(&state, &obj_id.to_hex()).await?;
        Ok((StatusCode::OK, Json(product)))
    }

    // ==========================================
    // 4. UPDATE (Put)
    // ==========================================
    pub async fn update(
        State(state): State<AppState>,
        Path(id): Path<String>,
        ValidatedJson(payload): ValidatedJson<ProductDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        // -------------------------------------------------------------
        // PLACE CUSTOM LOGIC / VERIFICATIONS HERE (e.g., checking permissions)
        // -------------------------------------------------------------

        product_services::update_product(&state, &obj_id.to_hex(), payload).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Product updated successfully" })),
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

        product_services::delete_product(&state, &obj_id.to_hex()).await?;
        Ok((
            StatusCode::OK,
            Json(serde_json::json!({ "message": "Product deleted successfully" })),
        ))
    }
}
