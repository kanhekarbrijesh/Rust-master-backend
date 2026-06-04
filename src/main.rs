use std::net::SocketAddr;

use axum::Router;

use crate::{
    configuration::config::get_configurations,
    services::logger_services::common_logging_services::AppLogger,
};

mod _utils;
mod configuration;
mod services;

#[tokio::main]
async fn main() {
    // load configurations
    let config = get_configurations();

    // setup logger
    AppLogger::new(config).log_app_config();

    //    setup routes
    let routes = Router::new();

    // setup listener
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // start server
    axum::serve(listener, routes).await.unwrap();
}
