pub mod private;
use axum::{Router, routing};

use crate::{infrastructure::app_state::AppState, routes::private::private_routes};

pub fn index_route() -> Router<AppState> {
    Router::new()
        .route("/", routing::get(|| async { "Hello, World!" }))
        .nest("/api/private", private_routes())
}
