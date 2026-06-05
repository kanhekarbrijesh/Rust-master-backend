use mongodb::Database;

use crate::infrastructure::_mongodb::mongodb::mongodb_connection;

#[derive(Clone)]
#[allow(dead_code)] // 👈 Temporary muzzle for the compiler warning
pub struct AppState {
    pub db: Database, // The live database instance
    pub mongodb_collections:
        crate::infrastructure::_mongodb::mongodb_collections::MongodbCollections,
}

impl AppState {
    pub async fn new(mongodb_uri: &str) -> Self {
        let db = mongodb_connection(mongodb_uri).await;
        let mongodb_collections =
            crate::infrastructure::_mongodb::mongodb_collections::MongodbCollections::new(
                db.clone(),
            );

        Self {
            db,
            mongodb_collections,
        }
    }
}
