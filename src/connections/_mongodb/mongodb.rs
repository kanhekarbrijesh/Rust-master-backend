use mongodb::{Client, Database};

use crate::_utils::constants::app_constants;

pub async fn mongodb_connection(mongodb_uri: &str) -> Database {
    // 1. Create a new MongoDB client (automatically parses mongodb+srv:// for Atlas)
    let client = Client::with_uri_str(mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB Atlas");

    // 2. Direct assignment! Return the database instance immediately
    client.database(app_constants::MONGODB_DB_NAME)
}
