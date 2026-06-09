use mongodb::bson::oid::ObjectId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::orders::order_types::OrderItem;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct OrderDto {
    #[garde(skip)]
    pub id: Option<String>,

    #[garde(length(min = 3))]
    pub order_number: String,

    #[garde(length(min = 1))]
    pub user_id: String,

    #[garde(length(min = 1))]
    pub items: Vec<OrderLineItemDto>,

    #[garde(range(min = 0))]
    pub total_in_cents: i64,

    #[garde(length(min = 3))]
    pub status: String,

    #[garde(pattern(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"))]
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct OrderLineItemDto {
    #[garde(length(min = 3))]
    pub sku: String,

    #[garde(range(min = 1))]
    pub quantity: i32,

    #[garde(range(min = 0))]
    pub price_in_cents: i64,
}

impl From<OrderDto> for OrderItem {
    fn from(dto: OrderDto) -> Self {
        Self {
            id: dto.id.and_then(|s| ObjectId::parse_str(s).ok()),
            order_number: dto.order_number,
            user_id: dto.user_id,
            items: dto
                .items
                .into_iter()
                .map(|it| crate::domain::orders::order_types::OrderLineItem {
                    sku: it.sku,
                    quantity: it.quantity,
                    price_in_cents: it.price_in_cents,
                })
                .collect(),
            total_in_cents: dto.total_in_cents,
            status: dto.status,
            created_at: dto.created_at,
        }
    }
}
