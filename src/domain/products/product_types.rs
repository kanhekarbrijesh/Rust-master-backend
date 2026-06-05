use mongodb::bson::oid::ObjectId;
// src/domain/products/product_types.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[allow(unexpected_cfgs)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct ProductItem {
    // ==========================================
    // 1. SYSTEM KEYS & AGNOSTIC IDENTIFIERS
    // ==========================================
    #[garde(skip)]
    #[schemars(skip)]
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    /// Use a string representation for ObjectId to avoid cross-crate serde/schemars
    /// trait-bound issues when multiple `bson` versions are present.
    // pub _id: Option<ObjectId>,
    pub id: Option<ObjectId>,

    /// STRONGLY TYPED REFERENCE (SKU):
    /// 🟢 Replaced \d with [0-9] and alpha classes for direct compilation
    #[garde(length(min = 8, max = 20), pattern(r"^[A-Z0-9]{3}-[A-Z0-9]{4}$"))]
    pub sku: String,

    // ==========================================
    // 2. TEXT PROCESSING & CAPTIONS
    // ==========================================
    #[garde(length(min = 2, max = 100))]
    pub item_name: String,

    #[garde(length(min = 10, max = 5000))]
    pub description: String,

    // ==========================================
    // 3. NUMERICAL BOUNDS & PRECISION
    // ==========================================
    #[garde(range(min = 0, max = 999999))]
    pub quantity: i32,

    #[garde(range(min = 100))]
    pub price_in_cents: i64,

    #[garde(range(min = 0, max = 100))]
    pub discount_percent: i32,

    // ==========================================
    // 4. METADATA, ARRAYS & FORMAT SPECIFICATIONS
    // ==========================================
    /// SUPPORT EMAIL:
    #[garde(length(max = 255), email)]
    pub supplier_email: String,

    /// SEARCH TAGGING ARRAYS:
    #[garde(length(min = 1, max = 10))]
    pub tags: Vec<String>,

    /// ISO TIMESTAMPS:
    /// 🟢 Fixed: Swapped shorthand \d for strict, native [0-9] digit arrays
    #[garde(pattern(r"^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"))]
    pub created_at: String,
}
