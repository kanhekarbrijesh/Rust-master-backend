use crate::_utils::{
    constants::{CONSTANTS, ENV_KEYS},
    functions::{get_env_var, load_env_file},
};

use super::Settings;

pub fn settings() -> Settings {
    load_env_file(".env.prod");

    Settings {
        app_name: CONSTANTS.app_name.to_string(),
        mongo_uri: get_env_var(ENV_KEYS.mongo_uri, ENV_KEYS.mongo_uri_default),
        port: get_env_var(ENV_KEYS.port, ENV_KEYS.port_default)
            .parse()
            .unwrap_or(8080),
    }
}
