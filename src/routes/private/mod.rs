use crate::infrastructure::app_state::AppState;

mod v1;

pub fn private_routes() -> axum::Router<AppState> {
    axum::Router::new().nest("/v1", v1::v1_routes())
}
