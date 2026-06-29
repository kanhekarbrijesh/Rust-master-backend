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
    pub postgresql_neon_pool_url: String,

    // ─── Storage ─────────────────────────────────────────────────────────
    /// Which storage backend to use: "local", "aws", "cloudflare"
    pub storage_provider: String,
    /// Base path on disk (used only when storage_provider = "local")
    pub storage_local_path: String,
    /// URL prefix for served files (used only when storage_provider = "local")
    pub storage_local_serve_prefix: String,

    // ─── Cloudflare R2 ───────────────────────────────────────────────────
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_endpoint: String,
    pub r2_bucket: String,
    pub r2_key_prefix: String,
    pub r2_public_url: String,

    // ─── AWS S3 ──────────────────────────────────────────────────────────
    pub aws_region: String,
    pub aws_bucket: String,
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
