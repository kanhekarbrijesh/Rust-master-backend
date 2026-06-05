use mongodb::{
    Collection,
    bson::{doc, oid::ObjectId},
};
use serde::{Serialize, de::DeserializeOwned};

// 1. Updated Trait Bounds to satisfy the compiler
#[derive(Clone)]
pub struct MongodbRepoV1<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
{
    collection: Collection<T>,
}

impl<T> MongodbRepoV1<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + Unpin,
{
    pub fn new(collection: Collection<T>) -> Self {
        Self { collection }
    }

    // CREATE
    // Change Result<ObjectId, String> to Result<ObjectId, mongodb::error::Error>
    // Return the actual error type, not a String
    pub async fn create(&self, item: T) -> Result<ObjectId, mongodb::error::Error> {
        // DO NOT use .map_err(|e| e.to_string()) here!
        let result = self.collection.insert_one(item).await?;

        result.inserted_id.as_object_id().ok_or_else(|| {
            // If you need to return a custom error here, wrap it in mongodb::error::Error
            mongodb::error::Error::from(std::io::Error::other("Failed to get inserted ID"))
        })
    }

    // READ
    pub async fn find_by_id(&self, id: &str) -> Result<Option<T>, String> {
        let oid = ObjectId::parse_str(id).map_err(|_| "Invalid ID format")?;
        // Removed 'None'
        self.collection
            .find_one(doc! { "_id": oid })
            .await
            .map_err(|e| e.to_string())
    }

    // UPDATE
    pub async fn update(
        &self,
        id: &str,
        update_doc: mongodb::bson::Document,
    ) -> Result<bool, String> {
        let oid = ObjectId::parse_str(id).map_err(|_| "Invalid ID format")?;
        // Removed 'None'
        let result = self
            .collection
            .update_one(doc! { "_id": oid }, update_doc)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.matched_count > 0)
    }

    // DELETE
    pub async fn delete(&self, id: &str) -> Result<bool, String> {
        let oid = ObjectId::parse_str(id).map_err(|_| "Invalid ID format")?;
        // Removed 'None'
        let result = self
            .collection
            .delete_one(doc! { "_id": oid })
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.deleted_count > 0)
    }
}
