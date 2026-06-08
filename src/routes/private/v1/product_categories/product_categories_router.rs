use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::product_categories::product_categories_controller::ProductCategoryController,
};
use axum::{Router, routing::post};

pub fn product_category_router() -> Router<AppState> {
    Router::new().route(
        "/",
        // Direct reference to the method, completely clean.
        post(ProductCategoryController::create),
    )
}
