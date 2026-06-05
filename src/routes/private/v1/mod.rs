use crate::{
    infrastructure::app_state::AppState, routes::private::v1::products::product_router::product_router,
};

pub mod products;

// 🟢 Specify that this Router carries AppState context
pub fn v1_routes() -> axum::Router<AppState> {
    axum::Router::new().nest("/products", product_router())
}
