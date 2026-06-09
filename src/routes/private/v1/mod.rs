use axum::Router;

use crate::{
    infrastructure::app_state::AppState,
    routes::private::v1::{
        product_categories::product_categories_router::product_category_router,
        products::product_router::product_router,
        user_roles::user_role_router::user_role_routes,
        orders::order_router::order_router,
    },
};

pub mod product_categories;
pub mod products;
pub mod user_roles;
pub mod orders;

// 🟢 Specify that this Router carries AppState context
pub fn v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/products", product_router())
        .nest("/product-categories", product_category_router())
        .nest("/user-roles", user_role_routes())
        .nest("/orders", order_router())
}
