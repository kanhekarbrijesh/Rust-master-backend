use crate::{
    _utils::constants::{app_environments, app_keys},
    configuration::{dev_config, local_config, prod_config, stage_config},
};

#[derive(Clone)]
pub struct Configs {
    pub app_name: String,
    pub mongo_uri: String,
    pub port: u16,
    pub current_env: String,
}

pub fn get_configurations() -> Configs {
    let env = std::env::var(app_keys::APP_ENV).unwrap_or_else(|_| app_environments::LOCAL.into());

    match env.as_str() {
        app_environments::DEV => dev_config::settings(),
        app_environments::PROD => prod_config::settings(),
        app_environments::STAGE => stage_config::settings(),
        _ => local_config::settings(),
    }
}
