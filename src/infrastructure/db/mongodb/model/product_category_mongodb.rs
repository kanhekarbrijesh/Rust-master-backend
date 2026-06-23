// use mongodb::Database;

// use crate::{
//     domain::product_categories::product_categories_repo::ProductCategoriesModel,
//     infrastructure::_mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
// };

// // step 1 : repo struct
// pub struct ProductCategoryRepo {
//     pub product_category_repo: MongodbRepoV1<ProductCategoriesModel>,
// }

// // step 2 : implement repo
// impl ProductCategoryRepo {
//     // step 3 : add contructor which ask for db instance of mongodb
//     pub fn new(db: Database) -> Self {
//         // step 4 : setup client for this collection and then wrap mongodb repo over this client
//         let product_category_collection =
//             db.collection::<ProductCategoriesModel>("productcategories");
//         let product_category_repo = MongodbRepoV1::new(product_category_collection);

//         Self {
//             product_category_repo,
//         }
//     }
// }

use mongodb::Database;

use crate::{
    domain::product_categories::product_categories_repo::ProductCategoriesModel,
    infrastructure::db::mongodb::repository::mongodb_repo_v1::MongodbRepoV1,
};

#[derive(Clone)]
pub struct ProductCategoryRepo {
    pub product_category_repo: MongodbRepoV1<ProductCategoriesModel>,
}

impl ProductCategoryRepo {
    pub fn new(db: Database) -> Self {
        Self {
            product_category_repo: MongodbRepoV1::new(
                db.collection::<ProductCategoriesModel>("productcategories")
                    .clone(),
            ),
        }
    }
}
