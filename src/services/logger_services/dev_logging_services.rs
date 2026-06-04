use std::net::Ipv4Addr;

use tracing::info;

use crate::{_utils::constants::app_constants, configuration::config::Configs};

pub struct DevLogger {
    log_config: Configs,
}

impl DevLogger {
    pub fn new(config: Configs) -> Self {
        DevLogger { log_config: config }
    }

    pub fn log_dev_config(&self) {
        info!("Running in Development Environment");
        info!("App Name: {}", self.log_config.app_name);
        info!("Mongo URI: {}", self.log_config.mongo_uri);
        info!("Port: {}", self.log_config.port);
        info!(
            "local endpoint is http::localhost:{}  or http://{}:{}",
            self.log_config.port,
            Ipv4Addr::from(app_constants::APP_IP),
            self.log_config.port
        );
    }
}
