mod sqlite;

use async_trait::async_trait;

use crate::errors::Result;
use crate::models::Url;

// Storage trait that defines the data access interface
#[async_trait]
pub trait Storage: Send + Sync + 'static {
    // Create a new shortened URL
    async fn create_url(&self, short_id: &str, original_url: &str) -> Result<Url>;

    // Get a URL by its short ID
    async fn get_url(&self, short_id: &str) -> Result<Url>;

    // Update the visit count for a URL
    async fn increment_visit(&self, short_id: &str) -> Result<()>;

    // Check if a URL with the given short ID exists
    async fn url_exists(&self, short_id: &str) -> Result<bool>;

    // Get all URLs with optional pagination
    async fn list_urls(&self, limit: usize, offset: usize) -> Result<Vec<Url>>;
}

#[derive(Debug, Clone)]
pub enum StorageType {
    Sqlite,
}

// Re-export SQLite implementation
pub use sqlite::SqliteStorage;

// Factory function to create a Storage implementation based on config
pub async fn create_storage(
    storage_type: &StorageType,
    database_url: &str,
) -> Result<Box<dyn Storage>> {
    match storage_type {
        StorageType::Sqlite => {
            let storage = SqliteStorage::new(database_url).await?;
            Ok(Box::new(storage))
        }
    }
}
