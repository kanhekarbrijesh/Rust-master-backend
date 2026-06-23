// src/services/domain_services/gallery_services.rs
use crate::{
    _utils::app_error::AppError,
    domain::gallery::{
        gallery_dto::{GalleryCreateDto, GalleryUpdateDto},
        gallery_types::GalleryItem,
    },
    infrastructure::app_state::AppState,
};
use mongodb::bson::doc;

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

pub async fn get_all_gallerys(state: &AppState) -> Result<Vec<GalleryItem>, AppError> {
    let gallerys = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .find()
        .await?; // The ? operator converts mongodb::error::Error to AppError automatically

    Ok(gallerys)
}

pub async fn get_gallery_by_id(state: &AppState, id: &str) -> Result<GalleryItem, AppError> {
    state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("gallery record not found".to_string()))
}

pub async fn update_gallery(
    state: &AppState,
    id: &str,
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
        .update(id, update_doc)
        .await?;
    if !updated {
        return Err(AppError::NotFound(
            "No matching gallery found to update".to_string(),
        ));
    }
    Ok(())
}

pub async fn delete_gallery(state: &AppState, id: &str) -> Result<(), AppError> {
    let deleted = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .delete(id)
        .await?;
    if !deleted {
        return Err(AppError::NotFound(
            "No matching gallery found to delete".to_string(),
        ));
    }
    Ok(())
}
