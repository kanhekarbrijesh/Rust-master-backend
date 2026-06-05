use mongodb::bson::oid::ObjectId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::products::product_types::ProductItem;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct ProductDto {
    #[garde(skip)] // The user doesn't provide this, so skip validation
    pub id: Option<String>,

    #[garde(length(min = 8, max = 20), pattern(r"^[A-Z0-9]{3}-[A-Z0-9]{4}$"))]
    pub sku: String,

    #[garde(length(min = 2, max = 100))]
    pub item_name: String,

    #[garde(length(min = 10, max = 5000))]
    pub description: String,

    #[garde(range(min = 0, max = 999999))]
    pub quantity: i32,

    #[garde(range(min = 100))]
    pub price_in_cents: i64,

    #[garde(range(min = 0, max = 100))]
    pub discount_percent: i32,

    #[garde(length(max = 255), email)]
    pub supplier_email: String,

    #[garde(length(min = 1, max = 10))]
    pub tags: Vec<String>,

    #[garde(pattern(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"))]
    pub created_at: String,
}

impl From<ProductDto> for ProductItem {
    fn from(dto: ProductDto) -> Self {
        Self {
            id: dto.id.and_then(|s| ObjectId::parse_str(s).ok()),
            sku: dto.sku,
            item_name: dto.item_name,
            description: dto.description,
            quantity: dto.quantity,
            price_in_cents: dto.price_in_cents,
            discount_percent: dto.discount_percent,
            supplier_email: dto.supplier_email,
            tags: dto.tags,
            created_at: dto.created_at,
        }
    }
}
