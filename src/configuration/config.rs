use crate::configuration::{dev_config, local_config, prod_config, stage_config};

pub struct Settings {
    pub app_name: String,
    pub mongo_uri: String,
    pub port: u16,
}

pub fn get_settings() -> Settings {
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "local".into());

    match env.as_str() {
        "dev" => dev_config::settings(),
        "prod" => prod_config::settings(),
        "stage" => stage_config::settings(),
        _ => local_config::settings(),
    }
}
