use std::net::SocketAddr;

use axum::Router;

use crate::{
    configuration::config::get_configurations, connections::app_state::AppState,
    routes::index_route, services::logger_services::common_logging_services::AppLogger,
};

// local crates
mod _utils;
mod configuration;
mod connections;
mod domain;
mod routes;
mod services;

#[tokio::main]
async fn main() {
    // load configurations
    let config = get_configurations();

    // setup logger
    AppLogger::new(config.clone()).log_app_config();

    // setting up app state
    let app_state = AppState::new(&config.mongo_uri.to_string()).await;

    //    setup routes
    let routes = Router::new().merge(index_route()).with_state(app_state);

    // setup listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // start server
    axum::serve(listener, routes).await.unwrap();
}
