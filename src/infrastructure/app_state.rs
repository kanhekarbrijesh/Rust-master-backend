use mongodb::Database;
use sqlx::PgPool;

use crate::{
    configuration::config::Configs,
    infrastructure::{
        _mongodb::mongodb::mongodb_connection, _postgresql::postgresql::connect_database_url,
    },
};

#[derive(Clone)]
#[allow(dead_code)] // 👈 Temporary muzzle for the compiler warning
pub struct AppState {
    pub db: Database, // The live database instance
    pub mongodb_collections:
        crate::infrastructure::_mongodb::mongodb_collections::MongodbCollections,
    // postgresql pgpool
    pub psql_pool: PgPool,
}

impl AppState {
    pub async fn new(config: Configs) -> Self {
        // --------------------------------------------------------------- start : mongodb setup --------
        // mongodb setup
        let mongodb_uri = &config.mongo_uri.clone();
        let db = mongodb_connection(mongodb_uri).await;
        let mongodb_collections =
            crate::infrastructure::_mongodb::mongodb_collections::MongodbCollections::new(
                db.clone(),
            );
        // --------------------------------------------------------------- end : mongodb setup --------

        // --------------------------------------------------------------- start : postgresql setup --------
        // Create a connection pool
        let postgresql_url = &config.postgresql_neon_pool_url;

        let psql_pool = connect_database_url(postgresql_url)
            .await
            .expect("Failed to connect to postgresql");
        // --------------------------------------------------------------- start : postgresql setup --------

        Self {
            db,
            mongodb_collections,
            psql_pool,
        }
    }
}
