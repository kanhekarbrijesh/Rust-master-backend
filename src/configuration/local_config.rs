use crate::_utils::{
    constants::{app_constants, app_keys},
    functions::{get_env_var, load_env_file},
};

use super::Configs;

pub fn settings() -> Configs {
    load_env_file(".env");

    Configs {
        app_name: app_constants::APP_NAME.to_string(),
        mongo_uri: get_env_var(app_keys::MONGO_URI, app_constants::MONGO_URI_DEFAULT),
        port: get_env_var(app_keys::PORT, app_constants::PORT_DEFAULT)
            .parse()
            .unwrap_or(8080),
        current_env: "local".into(),
        postgresql_neon_pool_url: get_env_var(
            app_keys::DATABASE_URL,
            app_constants::DATABASE_URL_DEFAULT,
        ),
    }
}
