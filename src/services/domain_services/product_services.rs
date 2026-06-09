// src/services/domain_services/product_services.rs
use crate::{
    _utils::app_error::AppError, domain::products::product_dto::ProductDto,
    domain::products::product_types::ProductItem, infrastructure::app_state::AppState,
};
use mongodb::bson::{doc, oid::ObjectId};

pub async fn create_product(state: &AppState, payload: ProductDto) -> Result<String, AppError> {
    let object_id = state
        .mongodb_collections
        .product_mongodb
        .product_repo
        .create(payload.into())
        .await?;
    Ok(object_id.to_hex())
}

pub async fn get_all_products(state: &AppState) -> Result<Vec<ProductItem>, AppError> {
    let products = state
        .mongodb_collections
        .product_mongodb
        .product_repo
        .find()
        .await?; // The ? operator converts mongodb::error::Error to AppError automatically

    Ok(products)
}

pub async fn get_product_by_id(state: &AppState, id: &str) -> Result<ProductItem, AppError> {
    state
        .mongodb_collections
        .product_mongodb
        .product_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Product record not found".to_string()))
}

pub async fn update_product(
    state: &AppState,
    id: &str,
    payload: ProductDto,
) -> Result<(), AppError> {
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

    let updated = state
        .mongodb_collections
        .product_mongodb
        .product_repo
        .update(id, update_doc)
        .await?;
    if !updated {
        return Err(AppError::NotFound(
            "No matching product found to update".to_string(),
        ));
    }
    Ok(())
}

pub async fn delete_product(state: &AppState, id: &str) -> Result<(), AppError> {
    let deleted = state
        .mongodb_collections
        .product_mongodb
        .product_repo
        .delete(id)
        .await?;
    if !deleted {
        return Err(AppError::NotFound(
            "No matching product found to delete".to_string(),
        ));
    }
    Ok(())
}
