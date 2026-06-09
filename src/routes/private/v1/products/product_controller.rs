// src/routes/v1/products/product_controller.rs
use crate::{
    _utils::validate_json::ValidatedJson,
    domain::products::product_dto::ProductDto,
    infrastructure::app_state::AppState,
    services::domain_services::product_services::{
        add_product_handler, delete_product_handler, get_all_products_handler,
        get_product_by_id_handler, update_product_handler,
    },
};
use axum::{
    extract::{Path, State},
    response::Response,
};
use futures::future::BoxFuture;

pub struct ProductController {
    pub create: fn(State<AppState>, ValidatedJson<ProductDto>) -> BoxFuture<'static, Response>,
    pub read_all: fn(State<AppState>) -> BoxFuture<'static, Response>,
    pub read_by_id: fn(State<AppState>, Path<String>) -> BoxFuture<'static, Response>,
    pub update: fn(
        State<AppState>,
        Path<String>,
        ValidatedJson<ProductDto>,
    ) -> BoxFuture<'static, Response>,
    pub delete: fn(State<AppState>, Path<String>) -> BoxFuture<'static, Response>,
}

impl ProductController {
    pub fn new() -> Self {
        Self {
            create: add_product_handler,
            read_all: get_all_products_handler,
            read_by_id: get_product_by_id_handler,
            update: update_product_handler,
            delete: delete_product_handler,
        }
    }
}
