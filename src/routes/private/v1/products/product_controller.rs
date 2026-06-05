// src/routes/v1/products/product_controller.rs
use crate::{
    connections::app_state::AppState,
    domain::products::product_types::ProductItem,
    services::domain_services::product_services::{
        add_product_handler, delete_product_handler, get_all_products_handler,
        get_product_by_id_handler, update_product_handler,
    },
};
use axum::{
    Json,
    extract::{Path, State},
    response::Response,
};
use futures::future::BoxFuture;

pub struct ProductController {
    pub create: fn(State<AppState>, Json<ProductItem>) -> BoxFuture<'static, Response>,
    pub read_all: fn(State<AppState>) -> BoxFuture<'static, Response>,
    pub read_by_id: fn(State<AppState>, Path<String>) -> BoxFuture<'static, Response>,
    pub update:
        fn(State<AppState>, Path<String>, Json<ProductItem>) -> BoxFuture<'static, Response>,
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
