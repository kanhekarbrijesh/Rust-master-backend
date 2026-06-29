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

        // ─── Storage defaults ────────────────────────────────────────────
        storage_provider: get_env_var("STORAGE_PROVIDER", "local"),
        storage_local_path: "./uploads".into(),
        storage_local_serve_prefix: "/uploads".into(),

        // ─── Cloudflare R2 ───────────────────────────────────────────────
        r2_access_key: get_env_var("R2_ACCESS_KEY", ""),
        r2_secret_key: get_env_var("R2_SECRET_KEY", ""),
        r2_endpoint: get_env_var("R2_ENDPOINT", ""),
        r2_bucket: get_env_var("R2_BUCKET", ""),
        r2_key_prefix: get_env_var("R2_KEY_PREFIX", "testing"),
        r2_public_url: get_env_var("R2_PUBLIC_URL", ""),

        // ─── AWS S3 ──────────────────────────────────────────────────────
        aws_region: get_env_var("AWS_REGION", "ap-south-1"),
        aws_bucket: get_env_var("AWS_S3_BUCKET", ""),
    }
}
