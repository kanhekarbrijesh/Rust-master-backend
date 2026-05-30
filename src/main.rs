use axum::{Router, routing::get};
use std::net::SocketAddr;

use crate::{_utils::constants::CONSTANTS, configuration::config::get_settings};

mod _utils;
mod configuration;

#[tokio::main]
async fn main() {
    // load config
    let config = get_settings();
    //  setup routes
    let app = Router::new().route("/", get(handler));

    // setup server
    let addr = SocketAddr::from((CONSTANTS.app_ip, config.port));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!(
        "Server for app {} running at http://{:?}:{}",
        CONSTANTS.app_name,
        std::net::Ipv4Addr::from(CONSTANTS.app_ip),
        config.port
    );

    // start server
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, World!"
}
