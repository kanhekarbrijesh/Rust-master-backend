// src/infrastructure/_mongodb/repository/product_mongodb.rs
use crate::{
    domain::products::product_types::ProductItem,
    infrastructure::_mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
};
use mongodb::Database;

#[derive(Clone)]
pub struct ProductMongodbRepo {
    pub product_repo: MongodbRepoV1<ProductItem>,
}

impl ProductMongodbRepo {
    pub fn new(db: Database) -> Self {
        Self {
            product_repo: MongodbRepoV1::new(db.collection::<ProductItem>("products")),
        }
    }
}
