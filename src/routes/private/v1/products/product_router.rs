// src/routes/v1/product_router.rs
use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::products::product_controller::ProductController,
};
use axum::{Router, routing::get};

pub fn product_router() -> Router<AppState> {
    Router::new()
        // 1. Root Collection Path
        .route(
            "/",
            get(ProductController::read_all).post(ProductController::create),
        )
        // 2. Resource Instance Path
        .route(
            "/{id}",
            get(ProductController::read_by_id)
                .put(ProductController::update)
                .delete(ProductController::delete),
        )
}
