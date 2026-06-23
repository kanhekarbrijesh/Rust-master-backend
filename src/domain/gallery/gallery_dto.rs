use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::gallery::gallery_types::GalleryItem;

// ─── CREATE DTO ──────────────────────────────────────────────────────────────
/// Used for creating a gallery item.
/// `url` is populated by the controller after file upload.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct GalleryCreateDto {
    #[garde(length(min = 1, max = 2048))]
    pub url: String,

    #[garde(length(min = 1, max = 50))]
    pub status: String,
}

// ─── GET / RESPONSE DTO ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GalleryDto {
    pub id: String,
    pub url: String,
    pub status: String,
}

// ─── UPDATE DTO ──────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, garde::Validate)]
pub struct GalleryUpdateDto {
    #[garde(skip)]
    pub id: String,

    #[garde(length(min = 1, max = 50))]
    pub status: String,
}

// ─── CONVERSIONS ─────────────────────────────────────────────────────────────
impl From<GalleryItem> for GalleryDto {
    fn from(item: GalleryItem) -> Self {
        Self {
            id: item.id.map(|oid| oid.to_hex()).unwrap_or_default(),
            url: item.url,
            status: item.status,
        }
    }
}

impl From<GalleryCreateDto> for GalleryItem {
    fn from(dto: GalleryCreateDto) -> Self {
        Self {
            id: None,
            url: dto.url,
            status: dto.status,
        }
    }
}
