use crate::{
    _utils::app_error::AppError, domain::orders::order_dto::OrderDto,
    domain::orders::order_types::OrderItem, infrastructure::app_state::AppState,
};
use mongodb::bson::{Bson, doc, oid::ObjectId, to_bson};

pub async fn create_order(state: &AppState, payload: OrderDto) -> Result<String, AppError> {
    let object_id = state
        .mongodb_collections
        .order_mongodb
        .order_repo
        .create(payload.into())
        .await?;
    Ok(object_id.to_hex())
}

pub async fn get_all_orders(state: &AppState) -> Result<Vec<OrderItem>, AppError> {
    let orders = state
        .mongodb_collections
        .order_mongodb
        .order_repo
        .find()
        .await?;
    Ok(orders)
}

pub async fn get_order_by_id(state: &AppState, id: &str) -> Result<OrderItem, AppError> {
    state
        .mongodb_collections
        .order_mongodb
        .order_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("Order record not found".to_string()))
}

pub async fn update_order(state: &AppState, id: &str, payload: OrderDto) -> Result<(), AppError> {
    let items_bson = to_bson(&payload.items).unwrap_or(Bson::Null);
    let update_doc = doc! {
        "$set": {
            "order_number": payload.order_number,
            "user_id": payload.user_id,
            "items": items_bson,
            "total_in_cents": payload.total_in_cents,
            "status": payload.status,
            "created_at": payload.created_at,
        }
    };

    let updated = state
        .mongodb_collections
        .order_mongodb
        .order_repo
        .update(id, update_doc)
        .await?;
    if !updated {
        return Err(AppError::NotFound(
            "No matching order found to update".to_string(),
        ));
    }
    Ok(())
}

pub async fn delete_order(state: &AppState, id: &str) -> Result<(), AppError> {
    let deleted = state
        .mongodb_collections
        .order_mongodb
        .order_repo
        .delete(id)
        .await?;
    if !deleted {
        return Err(AppError::NotFound(
            "No matching order found to delete".to_string(),
        ));
    }
    Ok(())
}

// ---------------------- Unit tests ----------------------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orders::order_dto::OrderLineItemDto;

    #[tokio::test]
    async fn dto_to_orderitem_conversion() {
        let dto = OrderDto {
            id: None,
            order_number: "ORD-0001".to_string(),
            user_id: "user123".to_string(),
            items: vec![OrderLineItemDto {
                sku: "SKU1".to_string(),
                quantity: 2,
                price_in_cents: 500,
            }],
            total_in_cents: 1000,
            status: "new".to_string(),
            created_at: "2026-01-01T00:00:00Z".to_string(),
        };

        let item: OrderItem = dto.into();
        assert_eq!(item.order_number, "ORD-0001");
        assert_eq!(item.items.len(), 1);
        assert_eq!(item.total_in_cents, 1000);
    }
}
