// 2. Create the "predefined object" as a constant
pub mod app_constants {
    pub const APP_NAME: &str = "MyStartupApp";
    pub const APP_IP: [u8; 4] = [127, 0, 0, 1];

    pub const MONGO_URI_DEFAULT: &str = "localhost:5432";
    pub const PORT_DEFAULT: &str = "8080";
}

// 3. Define another struct for environment variable keys

pub mod app_keys {
    pub const MONGO_URI: &str = "DATABASE_URL";
    pub const PORT: &str = "PORT";
    pub const APP_ENV: &str = "APP_ENV";
}

// hardcoded constants for environments
pub mod app_environments {
    pub const LOCAL: &str = "local";
    pub const DEV: &str = "development";
    pub const STAGE: &str = "staging";
    pub const PROD: &str = "production";
}
