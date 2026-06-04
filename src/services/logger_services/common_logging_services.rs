use tracing::info;

use crate::{
    _utils::constants::app_environments, configuration::config::Configs,
    services::logger_services::dev_logging_services::DevLogger,
};

pub struct AppLogger {
    log_config: Configs,
    dev_logger: DevLogger,
}

impl AppLogger {
    pub fn new(config: Configs) -> Self {
        // Initializes the global logging system to read from the RUST_LOG env variable
        // setup tracing
        let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()); // Falls back to info if RUST_LOG isn't set

        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
            .init();

        tracing::info!("Application started successfully!");

        let dev_logger = DevLogger::new(config.clone());
        AppLogger {
            log_config: config,
            dev_logger,
        }
    }

    fn is_local(&self) -> bool {
        self.log_config.current_env == app_environments::LOCAL
    }

    pub fn log_app_config(&self) {
        if self.is_local() {
            info!("Running in Local Environment");
            self.dev_logger.log_dev_config();
        } else {
            info!("Running in {} Environment", self.log_config.current_env);
        }
    }
}
