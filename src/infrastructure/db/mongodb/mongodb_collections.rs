use mongodb::Database;

use crate::infrastructure::db::mongodb::model::{
    gallery_mongodb::GalleryMongodbRepo, order_mongodb::OrderMongodbRepo,
    product_category_mongodb::ProductCategoryRepo, product_mongodb::ProductMongodbRepo,
};

#[derive(Clone)]
pub struct MongodbCollections {
    pub product_mongodb: ProductMongodbRepo,
    pub prooduct_category: ProductCategoryRepo,
    pub order_mongodb: OrderMongodbRepo,
    pub gallery_mongodb: GalleryMongodbRepo,
}

impl MongodbCollections {
    pub fn new(db: Database) -> Self {
        let product_mongodb = ProductMongodbRepo::new(db.clone());
        let prooduct_category = ProductCategoryRepo::new(db.clone());
        let order_mongodb = OrderMongodbRepo::new(db.clone());
        let gallery_mongodb = GalleryMongodbRepo::new(db.clone());

        Self {
            product_mongodb,
            prooduct_category,
            order_mongodb,
            gallery_mongodb,
        }
    }
}
