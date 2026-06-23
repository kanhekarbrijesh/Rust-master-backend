use crate::{
    domain::orders::order_types::OrderItem,
    infrastructure::db::mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
};
use mongodb::Database;

#[derive(Clone)]
pub struct OrderMongodbRepo {
    pub order_repo: MongodbRepoV1<OrderItem>,
}

impl OrderMongodbRepo {
    pub fn new(db: Database) -> Self {
        Self {
            order_repo: MongodbRepoV1::new(db.collection::<OrderItem>("orders")),
        }
    }
}
