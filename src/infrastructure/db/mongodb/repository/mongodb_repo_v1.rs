use futures::stream::TryStreamExt;
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

    pub async fn find(&self) -> Result<Vec<T>, mongodb::error::Error> {
        // 1. Get the cursor
        let cursor = self.collection.find(doc! {}).await?;

        // 2. Turn the cursor into a stream and collect it into a Vec
        // This automatically handles the Result<T, Error> internally
        let items: Vec<T> = cursor.try_collect().await?;

        Ok(items)
    }

    // READ
    pub async fn find_by_id(&self, id: &str) -> Result<Option<T>, mongodb::error::Error> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| mongodb::error::Error::custom("Invalid ID format"))?;
        // Removed 'None'
        self.collection.find_one(doc! { "_id": oid }).await
    }

    // UPDATE
    pub async fn update(
        &self,
        id: &str,
        update_doc: mongodb::bson::Document,
    ) -> Result<bool, mongodb::error::Error> {
        let oid = ObjectId::parse_str(id)
            .map_err(|_| mongodb::error::Error::custom("Invalid Id Format"))?;
        // Removed 'None'
        let result = self
            .collection
            .update_one(doc! { "_id": oid }, update_doc)
            .await?;

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
