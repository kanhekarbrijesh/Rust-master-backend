use axum::{Router, routing::get};

use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::gallery::gallery_controller::GalleryController,
};

pub fn gallery_router() -> Router<AppState> {
    Router::new()
        // 1. Root Collection Path
        .route(
            "/",
            get(GalleryController::read_all).post(GalleryController::create),
        )
        // 2. Resource Instance Path
        .route(
            "/{id}",
            get(GalleryController::read_by_id)
                .put(GalleryController::update)
                .delete(GalleryController::delete),
        )
}
