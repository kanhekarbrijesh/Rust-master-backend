// src/infrastructure/_mongodb/repository/product_mongodb.rs
use crate::{
    domain::gallery::gallery_types::GalleryItem,
    infrastructure::db::mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
};
use mongodb::Database;

#[derive(Clone)]
pub struct GalleryMongodbRepo {
    pub gallery_repo: MongodbRepoV1<GalleryItem>,
}

impl GalleryMongodbRepo {
    pub fn new(db: Database) -> Self {
        Self {
            gallery_repo: MongodbRepoV1::new(db.collection::<GalleryItem>("galleries")),
        }
    }
}
