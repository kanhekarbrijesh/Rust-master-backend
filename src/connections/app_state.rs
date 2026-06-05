use mongodb::Database;

use crate::connections::_mongodb::mongodb::mongodb_connection;

#[derive(Clone)]
#[allow(dead_code)] // 👈 Temporary muzzle for the compiler warning
pub struct AppState {
    pub db: Database, // The live database instance
}

impl AppState {
    pub async fn new(mongodb_uri: &str) -> Self {
        let db = mongodb_connection(mongodb_uri).await;
        Self { db }
    }
}
