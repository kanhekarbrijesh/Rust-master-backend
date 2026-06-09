use crate::{
    _utils::validate_json::ValidatedJson,
    domain::products::{product_dto::ProductDto, product_types::ProductItem},
    infrastructure::app_state::AppState,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use garde::Validate;
use mongodb::bson::{doc, oid::ObjectId};
use tracing::{error, info};

// ==========================================
// 1. CREATE (Add Product)
// ==========================================
pub fn add_product_handler(
    State(state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<ProductDto>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        // ✅ CORRECT: Convert DTO to Entity before sending to the database
        let product_item: ProductItem = payload.into();

        match state
            .mongodb_collections
            .product_mongodb
            .product_repo
            .create(product_item)
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

// ==========================================
// 2. READ ALL (Get All Products)
// ==========================================
pub fn get_all_products_handler(State(state): State<AppState>) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        match state
            .mongodb_collections
            .product_mongodb
            .product_repo
            .find()
            .await
        {
            Ok(products) => (StatusCode::OK, Json(products)).into_response(),
            Err(err) => {
                error!("Failed to fetch products from MongoDB: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to retrieve records",
                )
                    .into_response()
            }
        }
    })
}

// ==========================================
// 3. READ ONE (Get Product By ID)
// ==========================================
pub fn get_product_by_id_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        let obj_id = match ObjectId::parse_str(&id) {
            Ok(oid) => oid,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID string format").into_response(),
        };

        match state
            .mongodb_collections
            .product_mongodb
            .product_repo
            .find_by_id(&obj_id.to_string())
            .await
        {
            Ok(Some(product)) => (StatusCode::OK, Json(product)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "Product record not found").into_response(),
            Err(err) => {
                error!("MongoDB find_one error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database lookup failure").into_response()
            }
        }
    })
}

// ==========================================
// 4. UPDATE (Update Product By ID)
// ==========================================
pub fn update_product_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ValidatedJson(payload): ValidatedJson<ProductDto>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        let obj_id = match ObjectId::parse_str(&id) {
            Ok(oid) => oid,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID string format").into_response(),
        };

        let update_doc = doc! {
            "$set": {
                "sku": payload.sku,
                "item_name": payload.item_name,
                "description": payload.description,
                "quantity": payload.quantity,
                "price_in_cents": payload.price_in_cents,
                "discount_percent": payload.discount_percent,
                "supplier_email": payload.supplier_email,
                "tags": payload.tags,
                "created_at": payload.created_at,
            }
        };

        match state
            .mongodb_collections
            .product_mongodb
            .product_repo
            .update(&obj_id.to_string(), update_doc)
            .await
        {
            Ok(result) => {
                if !result {
                    return (StatusCode::NOT_FOUND, "No matching product found to update")
                        .into_response();
                }

                info!("Successfully updated product ID: {}", id);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({ "message": "Product updated successfully" })),
                )
                    .into_response()
            }
            Err(err) => {
                if let mongodb::error::ErrorKind::Command(ref reply) = *err.kind {
                    if reply.code == 11000 {
                        return (
                            StatusCode::CONFLICT,
                            Json(serde_json::json!({ "error": "Modification failed. Value breaks unique field constraints." })),
                        ).into_response();
                    }
                    if reply.code == 121 {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({ "error": "Schema validation failed on database engine level." }))
                        ).into_response();
                    }
                }

                tracing::error!("MongoDB update_one error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database modification failure",
                )
                    .into_response()
            }
        }
    })
}

// ==========================================
// 5. DELETE (Remove Product By ID)
// ==========================================
pub fn delete_product_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        let obj_id = match ObjectId::parse_str(&id) {
            Ok(oid) => oid,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID string format").into_response(),
        };

        match state
            .mongodb_collections
            .product_mongodb
            .product_repo
            .delete(&obj_id.to_string())
            .await
        {
            Ok(result) => {
                if !result {
                    return (StatusCode::NOT_FOUND, "No matching product found to delete")
                        .into_response();
                }
                info!("Successfully deleted product record ID: {}", id);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({ "message": "Product deleted successfully" })),
                )
                    .into_response()
            }
            Err(err) => {
                error!("MongoDB delete_one error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database deletion failure",
                )
                    .into_response()
            }
        }
    })
}
