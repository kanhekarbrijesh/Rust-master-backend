use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use tracing::error;

use crate::{
    domain::product_categories::product_categories_repo::ProductCategoriesModel,
    infrastructure::app_state::AppState,
};

pub fn add_product_category_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductCategoriesModel>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        match state
            .mongodb_collections
            .prooduct_category
            .product_category_repo
            .create(payload.into())
            .await
        {
            Ok(object_id) => {
                let generated_id = object_id.to_hex();
                (
                    StatusCode::CREATED,
                    Json(serde_json::json!({ "id": generated_id })),
                )
                    .into_response()
            }
            Err(err) => {
                // Now 'err' IS a mongodb::error::Error.
                // We look inside 'err.kind' to check if it's a Command error.
                let _: () = if let mongodb::error::ErrorKind::Command(ref command_error) = *err.kind
                    && command_error.code == 11000
                {
                    return (
                        StatusCode::CONFLICT,
                        Json(serde_json::json!({ "error": "Duplicate product" })),
                    )
                        .into_response();
                };

                error!("MongoDB write error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database failure").into_response()
            }
        }
    })
}
