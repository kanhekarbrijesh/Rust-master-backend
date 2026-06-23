use axum::{Router, routing::get};

use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::users::user_controller::UserController,
};

pub fn user_router() -> Router<AppState> {
    Router::new()
        // 1. Root Collection Path
        .route(
            "/",
            get(UserController::read_all).post(UserController::create),
        )
        // 2. Resource Instance Path
        .route(
            "/{id}",
            get(UserController::read_by_id)
                .put(UserController::update)
                .delete(UserController::delete),
        )
}
