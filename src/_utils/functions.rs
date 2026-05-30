use std::path::Path;

pub fn get_env_var(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn load_env_file(file_name: &str) {
    // 1. Try to load the specific environment file
    // 2. Fallback to .env if the specific one doesn't exist
    if Path::new(file_name).exists() {
        dotenvy::from_filename(file_name).ok();
    } else {
        dotenvy::from_filename(".env").ok();
    }
}
