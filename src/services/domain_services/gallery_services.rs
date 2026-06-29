// src/services/domain_services/gallery_services.rs
use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    _utils::app_error::AppError,
    domain::gallery::{
        gallery_dto::{GalleryCreateDto, GalleryUpdateDto},
        gallery_types::GalleryItem,
    },
    infrastructure::{app_state::AppState, storage::storage_util},
};

pub async fn create_gallery(
    state: &AppState,
    payload: GalleryCreateDto,
) -> Result<String, AppError> {
    let object_id = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .create(payload.into())
        .await?;
    Ok(object_id.to_hex())
}

pub async fn get_all_galleries(state: &AppState) -> Result<Vec<GalleryItem>, AppError> {
    let items = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .find()
        .await?;
    Ok(items)
}

pub async fn get_gallery_by_id(state: &AppState, id: ObjectId) -> Result<GalleryItem, AppError> {
    state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .find_by_object_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("gallery record not found".to_string()))
}

pub async fn update_gallery(
    state: &AppState,
    id: ObjectId,
    payload: GalleryUpdateDto,
) -> Result<(), AppError> {
    let update_doc = doc! {
        "$set": {
            "status": payload.status,
        }
    };

    let updated = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .update_by_object_id(id, update_doc)
        .await?;
    if !updated {
        return Err(AppError::NotFound(
            "No matching gallery found to update".to_string(),
        ));
    }
    Ok(())
}

pub async fn delete_gallery(
    state: &AppState,
    serve_prefix: &str,
    id: ObjectId,
) -> Result<(), AppError> {
    // ── Fetch the gallery item first to get the file URL ──────────────────
    let item = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .find_by_object_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("No matching gallery found to delete".to_string()))?;

    // ── Delete the file from storage (best-effort) ────────────────────────
    let file_key = storage_util::storage_key_from_url(&item.url, serve_prefix);
    storage_util::delete_file_quietly(&*state.storage, &file_key).await;

    // ── Delete from DB ────────────────────────────────────────────────────
    state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .delete_by_object_id(id)
        .await?;

    Ok(())
}
