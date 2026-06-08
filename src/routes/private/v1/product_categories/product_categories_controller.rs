use crate::{
    domain::product_categories::product_categories_repo::{self, ProductCategoriesModel},
    infrastructure::app_state::AppState,
    services::domain_services::product_categories_services,
};
use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};

pub struct ProductCategoryController;

impl ProductCategoryController {
    // A clean async method that delegates directly to your handler service layer
    pub async fn create(state: State<AppState>, payload: Json<ProductCategoriesModel>) -> Response {
        product_categories_services::add_product_category_handler(state, payload).await
    }
    pub async fn update(
        state: State<AppState>,
        id: Path<String>,
        payload: Json<ProductCategoriesModel>,
    ) -> Response {
        product_categories_services::update_product_category_handler(state, id, payload).await
    }
}
