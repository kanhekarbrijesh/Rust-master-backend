// src/routes/v1//product_router.rs
use crate::{
    connections::app_state::AppState,
    routes::private::v1::products::product_controller::ProductController,
};
use axum::{
    Router,
    routing::{delete, get, post, put},
};

pub fn product_router() -> Router<AppState> {
    let controller = ProductController::new();

    Router::new()
        // CREATE
        .route(
            "/",
            post(move |state, payload| (controller.create)(state, payload)),
        )
        // READ ALL
        .route("/", get(move |state| (controller.read_all)(state)))
        // READ BY ID
        .route(
            "/{id}",
            get(move |state, path| (controller.read_by_id)(state, path)),
        )
        // UPDATE BY ID
        .route(
            "/{id}",
            put(move |state, path, payload| (controller.update)(state, path, payload)),
        )
        // DELETE BY ID
        .route(
            "/{id}",
            delete(move |state, path| (controller.delete)(state, path)),
        )
}
