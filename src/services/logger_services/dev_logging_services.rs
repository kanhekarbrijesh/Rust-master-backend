use std::net::Ipv4Addr;

use tracing::info;

use crate::{
    _utils::{
        constants::app_constants,
        security::{SecretMaskStrategy, SecureLogUtil},
    },
    configuration::config::Configs,
};

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

        // 1. Secure MongoDB URI (Database URL parsing strategy)
        let safe_mongo = SecureLogUtil::mask_value(
            "MONGO_URI",
            &self.log_config.mongo_uri,
            SecretMaskStrategy::DatabaseUri,
        );
        tracing::info!("Database Connection mongodb : {}", safe_mongo);

        let safe_postgresql = SecureLogUtil::mask_value(
            "MONGO_URI",
            &self.log_config.postgresql_neon_pool_url,
            SecretMaskStrategy::DatabaseUri,
        );
        tracing::info!("Database Connection postgresql : {}", safe_postgresql);

        info!("Port: {}", self.log_config.port);
        info!(
            "local endpoint is http://localhost:{}  or http://{}:{}",
            self.log_config.port,
            Ipv4Addr::from(app_constants::APP_IP),
            self.log_config.port
        );
    }
}
