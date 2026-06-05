pub mod private;
use crate::{infrastructure::app_state::AppState, routes::private::private_routes};

pub fn index_route() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .nest("/api/private", private_routes())
}
