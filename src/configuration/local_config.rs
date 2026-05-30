use crate::_utils::{
    constants::{CONSTANTS, ENV_KEYS},
    functions::get_env_var,
};

use super::Settings;

pub fn settings() -> Settings {
    // Optional: Load .env.local here if you still want .env file support
    dotenvy::from_filename(".env").ok();

    Settings {
        app_name: CONSTANTS.app_name.to_string(),
        mongo_uri: get_env_var(ENV_KEYS.mongo_uri, ENV_KEYS.mongo_uri_default),
        port: get_env_var(ENV_KEYS.port, ENV_KEYS.port_default)
            .parse()
            .unwrap_or(8080),
    }
}
