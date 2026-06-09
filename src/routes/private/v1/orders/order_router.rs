use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::orders::order_controller::OrderController,
};
use axum::{Router, routing::get};

pub fn order_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(OrderController::read_all).post(OrderController::create),
        )
        .route(
            "/{id}",
            get(OrderController::read_by_id)
                .put(OrderController::update)
                .delete(OrderController::delete),
        )
}
