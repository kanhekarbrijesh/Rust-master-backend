// src/routes/private/v1/user_roles/user_role_router.rs

use crate::AppState;
use crate::routes::private::v1::user_roles::user_role_controller::{
    create_role_handler, get_role_handler, update_role_handler,
};
use axum::{
    Router,
    routing::{get, post},
};

pub fn user_role_routes() -> Router<AppState> {
    // 🌟 Typed directly with <AppState> to pass Axum 0.8 route validation
    Router::<AppState>::new()
        .route("/", post(create_role_handler))
        .route("/{id}", get(get_role_handler).put(update_role_handler))
}
