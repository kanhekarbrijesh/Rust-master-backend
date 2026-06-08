use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

use crate::{
    domain::product_categories::product_categories_repo::ProductCategoriesModel,
    infrastructure::app_state::AppState,
};

pub async fn add_product_category_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductCategoriesModel>,
) -> Response {
    match state
        .mongodb_collections
        .prooduct_category // Note: Watch out for this typo in your struct definition!
        .product_category_repo
        .create(payload)
        .await
    {
        // Success
        Ok(object_id) => {
            let generated_id = object_id.to_hex();
            (
                StatusCode::CREATED,
                Json(serde_json::json!({ "id": generated_id })),
            )
                .into_response()
        }

        // Error handling
        Err(err) => {
            // 1. Check if it's a MongoDB Command error with code 11000
            if let mongodb::error::ErrorKind::Command(ref command_error) = *err.kind
                && command_error.code == 11000
            {
                return (
                    StatusCode::CONFLICT,
                    Json(serde_json::json!({ "error": "Duplicate product" })),
                )
                    .into_response();
            }

            // 2. Fallback for any other unexpected database error
            error!("MongoDB write error: {:?}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database failure").into_response()
        }
    }
}
