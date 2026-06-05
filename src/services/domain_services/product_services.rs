use crate::{connections::app_state::AppState, domain::products::product_types::ProductItem};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::future::BoxFuture;
use futures::stream::TryStreamExt;
use garde::Validate;
use mongodb::bson::{doc, oid::ObjectId};

// ==========================================
// 1. CREATE (Add Product)
// ==========================================
pub fn add_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<ProductItem>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        // 🛑 EXACT FIX: Parentheses are completely empty here now!
        if let Err(report) = payload.validate() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Validation failed", "details": report.to_string() })),
            )
                .into_response();
        }

        let collection = state.db.collection::<ProductItem>("products");

        match collection.insert_one(payload).await {
            Ok(result) => {
                let generated_id = result.inserted_id.as_object_id().unwrap().to_hex();
                tracing::info!(
                    "Successfully inserted product record with ID: {}",
                    generated_id
                );

                (
                    StatusCode::CREATED,
                    Json(serde_json::json!({ "id": generated_id })),
                )
                    .into_response()
            }
            Err(err) => {
                if let mongodb::error::ErrorKind::Command(ref reply) = *err.kind
                    && reply.code == 11000
                {
                    return (
                            StatusCode::CONFLICT,
                            Json(serde_json::json!({ "error": "A product with this unique constraint identifier already exists." })),
                        ).into_response();
                }

                tracing::error!("MongoDB write error encountered: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database write failure").into_response()
            }
        }
    })
}

// ==========================================
// 2. READ ALL (Get All Products)
// ==========================================
pub fn get_all_products_handler(State(state): State<AppState>) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        let collection = state.db.collection::<ProductItem>("products");

        match collection.find(doc! {}).await {
            Ok(mut cursor) => {
                let mut products = Vec::new();

                while let Ok(Some(product)) = cursor.try_next().await {
                    products.push(product);
                }

                (StatusCode::OK, Json(products)).into_response()
            }
            Err(err) => {
                tracing::error!("Failed to fetch products from MongoDB: {:?}", err);
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

        let collection = state.db.collection::<ProductItem>("products");

        match collection.find_one(doc! { "_id": obj_id }).await {
            Ok(Some(product)) => (StatusCode::OK, Json(product)).into_response(),
            Ok(None) => (StatusCode::NOT_FOUND, "Product record not found").into_response(),
            Err(err) => {
                tracing::error!("MongoDB find_one error: {:?}", err);
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
    Json(payload): Json<ProductItem>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move {
        // 🛑 EXACT FIX: Parentheses are completely empty here now!
        if let Err(report) = payload.validate() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Validation failed", "details": report.to_string() })),
            )
                .into_response();
        }

        let obj_id = match ObjectId::parse_str(&id) {
            Ok(oid) => oid,
            Err(_) => return (StatusCode::BAD_REQUEST, "Invalid ID string format").into_response(),
        };

        let collection = state.db.collection::<ProductItem>("products");

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

        match collection
            .update_one(doc! { "_id": obj_id }, update_doc)
            .await
        {
            Ok(result) => {
                if result.matched_count == 0 {
                    return (StatusCode::NOT_FOUND, "No matching product found to update")
                        .into_response();
                }

                tracing::info!("Successfully updated product ID: {}", id);
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

        let collection = state.db.collection::<ProductItem>("products");

        match collection.delete_one(doc! { "_id": obj_id }).await {
            Ok(result) => {
                if result.deleted_count == 0 {
                    return (StatusCode::NOT_FOUND, "No matching product found to delete")
                        .into_response();
                }
                tracing::info!("Successfully deleted product record ID: {}", id);
                (
                    StatusCode::OK,
                    Json(serde_json::json!({ "message": "Product deleted successfully" })),
                )
                    .into_response()
            }
            Err(err) => {
                tracing::error!("MongoDB delete_one error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database deletion failure",
                )
                    .into_response()
            }
        }
    })
}
