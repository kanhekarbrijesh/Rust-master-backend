use mongodb::Database;

use crate::{
    domain::products::product_types::ProductItem,
    infrastructure::_mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
};

#[derive(Clone)]
pub struct ProductMongodbRepo {
    // pub product_collection: Collection<ProductItem>,
    pub product_repo: MongodbRepoV1<ProductItem>,
}

impl ProductMongodbRepo {
    pub fn new(db: Database) -> Self {
        let product_collection = db.collection::<ProductItem>("products");
        let product_repo = MongodbRepoV1::new(product_collection.clone());

        Self {
            // product_collection,
            product_repo,
        }
    }
}
