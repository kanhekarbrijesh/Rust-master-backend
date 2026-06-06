use mongodb::Database;

use crate::infrastructure::_mongodb::model::{
    product_category_mongodb::ProductCategoryRepo, product_mongodb::ProductMongodbRepo,
};

#[derive(Clone)]
pub struct MongodbCollections {
    pub product_mongodb: ProductMongodbRepo,
    pub prooduct_category: ProductCategoryRepo,
}

impl MongodbCollections {
    pub fn new(db: Database) -> Self {
        let product_mongodb = ProductMongodbRepo::new(db.clone());
        let prooduct_category = ProductCategoryRepo::new(db);

        Self {
            product_mongodb,
            prooduct_category,
        }
    }
}
