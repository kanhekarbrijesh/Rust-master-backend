use crate::{
    _utils::{app_error::AppError, validate_json::ValidatedJson},
    domain::orders::order_dto::OrderDto,
    infrastructure::app_state::AppState,
    services::domain_services::order_services,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use mongodb::bson::oid::ObjectId;

pub struct OrderController;

impl OrderController {
    pub async fn create(
        State(state): State<AppState>,
        ValidatedJson(payload): ValidatedJson<OrderDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let generated_id = order_services::create_order(&state, payload).await?;
        Ok((StatusCode::CREATED, Json(serde_json::json!({ "id": generated_id }))))
    }

    pub async fn read_all(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
        let orders = order_services::get_all_orders(&state).await?;
        Ok((StatusCode::OK, Json(orders)))
    }

    pub async fn read_by_id(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        let order = order_services::get_order_by_id(&state, &obj_id.to_hex()).await?;
        Ok((StatusCode::OK, Json(order)))
    }

    pub async fn update(
        State(state): State<AppState>,
        Path(id): Path<String>,
        ValidatedJson(payload): ValidatedJson<OrderDto>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        order_services::update_order(&state, &obj_id.to_hex(), payload).await?;
        Ok((StatusCode::OK, Json(serde_json::json!({ "message": "Order updated successfully" }))))
    }

    pub async fn delete(
        State(state): State<AppState>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, AppError> {
        let obj_id = ObjectId::parse_str(&id)
            .map_err(|_| AppError::BadRequest("Invalid ID string format".to_string()))?;

        order_services::delete_order(&state, &obj_id.to_hex()).await?;
        Ok((StatusCode::OK, Json(serde_json::json!({ "message": "Order deleted successfully" }))))
    }
}
