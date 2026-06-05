use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// Remove JsonSchema and garde::Validate from here.
// These belong in your DTOs, not your Database Entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub sku: String,
    pub item_name: String,
    pub description: String,
    pub quantity: i32,
    pub price_in_cents: i64,
    pub discount_percent: i32,
    pub supplier_email: String,
    pub tags: Vec<String>,
    pub created_at: String,
}
