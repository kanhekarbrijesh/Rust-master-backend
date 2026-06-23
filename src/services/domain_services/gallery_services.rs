// src/services/domain_services/gallery_services.rs
use mongodb::bson::{doc, oid::ObjectId};

use crate::{
    _utils::app_error::AppError,
    domain::gallery::{
        gallery_dto::{GalleryCreateDto, GalleryUpdateDto},
        gallery_types::GalleryItem,
    },
    infrastructure::app_state::AppState,
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

pub async fn delete_gallery(state: &AppState, id: ObjectId) -> Result<(), AppError> {
    let deleted = state
        .mongodb_collections
        .gallery_mongodb
        .gallery_repo
        .delete_by_object_id(id)
        .await?;
    if !deleted {
        return Err(AppError::NotFound(
            "No matching gallery found to delete".to_string(),
        ));
    }
    Ok(())
}
