// 1. Define the struct to act as your object
pub struct AppConstants {
    pub app_name: &'static str,
    pub app_ip: [u8; 4],
}

// 2. Create the "predefined object" as a constant
pub const CONSTANTS: AppConstants = AppConstants {
    app_name: "MyStartupApp",
    app_ip: [127, 0, 0, 1],
};

pub struct EnvKeys {
    pub mongo_uri: &'static str,
    pub mongo_uri_default: &'static str,
    pub port: &'static str,
    pub port_default: &'static str,
}

pub const ENV_KEYS: EnvKeys = EnvKeys {
    mongo_uri: "DATABASE_URL",
    mongo_uri_default: "localhost:5432",
    port: "PORT",
    port_default: "8080",
};
