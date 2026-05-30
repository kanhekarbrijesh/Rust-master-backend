use axum::Router;
use std::net::SocketAddr;

use crate::{_utils::constants::CONSTANTS, configuration::config::get_settings};

mod _utils;
mod configuration;

#[tokio::main]
async fn main() {
    // load config
    let config = get_settings();
    //  setup routes
    let app = Router::new();

    // setup server
    let addr = SocketAddr::from((CONSTANTS.app_ip, config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    // start server
    axum::serve(listener, app).await.unwrap();
}
