use mongodb::Database;

use crate::infrastructure::_mongodb::model::product_mongodb::ProductMongodbRepo;

#[derive(Clone)]
pub struct MongodbCollections {
    pub product_mongodb: ProductMongodbRepo,
}

impl MongodbCollections {
    pub fn new(db: Database) -> Self {
        let product_mongodb = ProductMongodbRepo::new(db);

        Self { product_mongodb }
    }
}
