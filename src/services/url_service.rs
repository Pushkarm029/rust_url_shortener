use crate::config::Config;
use crate::errors::{AppError, Result};
use crate::models::UnixNanos;
use crate::models::{Url, UrlResponse};
use crate::storage::Storage;
use crate::utils;

/// Service that handles URL shortening business logic
///
/// This service encapsulates all operations related to URL shortening,
/// acting as a middleware between the HTTP handlers and storage layer.
/// It enforces business rules and validation logic.
pub struct UrlService {
    storage: Box<dyn Storage>,
    config: Config,
}

impl UrlService {
    /// Creates a new UrlService with the given storage implementation and configuration
    pub fn new(storage: Box<dyn Storage>, config: Config) -> Self {
        Self { storage, config }
    }

    /// Shortens a URL, either with a provided custom ID or a generated one
    ///
    /// # Arguments
    /// * `original_url` - The URL to shorten
    /// * `custom_id` - Optional custom identifier for the shortened URL
    ///
    /// # Returns
    /// * `Result<UrlResponse>` - The shortened URL information or an error
    pub async fn shorten_url(
        &self,
        original_url: &str,
        custom_id: Option<String>,
    ) -> Result<UrlResponse> {
        // Validate the URL
        if !utils::validate_url(original_url) {
            return Err(AppError::InvalidUrl(format!(
                "Invalid URL: {}. URL must start with http:// or https://",
                original_url
            )));
        }

        // Generate or use the provided short ID
        let short_id = match custom_id {
            Some(id) => {
                // Validate custom ID
                if !utils::validate_short_id(&id) {
                    return Err(AppError::ValidationError(
                        "Custom ID must contain only alphanumeric characters".to_string(),
                    ));
                }

                // Check if the ID is already in use
                if self.storage.url_exists(&id).await? {
                    return Err(AppError::ValidationError(format!(
                        "Short ID '{}' is already in use",
                        id
                    )));
                }

                id
            }
            None => {
                // Generate a random short ID
                let mut attempts = 0;
                let max_attempts = 5;
                let mut short_id;

                loop {
                    short_id = utils::generate_short_id(self.config.short_url_length);
                    if !self.storage.url_exists(&short_id).await? {
                        break;
                    }

                    attempts += 1;
                    if attempts >= max_attempts {
                        return Err(AppError::InternalError(
                            "Failed to generate a unique short ID".to_string(),
                        ));
                    }
                }

                short_id
            }
        };

        // Store the URL
        let url = self.storage.create_url(&short_id, original_url).await?;

        // Return the response
        Ok(UrlResponse {
            short_id: url.short_id.clone(),
            original_url: url.original_url.clone(),
        })
    }

    pub async fn get_url(&self, short_id: &str) -> Result<String> {
        let url = self.storage.get_url(short_id).await?;
        self.storage.increment_visit(short_id).await?;

        Ok(url.original_url)
    }

    pub async fn get_url_stats(&self, short_id: &str) -> Result<Url> {
        self.storage.get_url(short_id).await
    }

    pub async fn list_urls(&self, limit: usize, offset: usize) -> Result<Vec<Url>> {
        self.storage.list_urls(limit, offset).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    // Mock storage implementation for testing
    struct MockStorage {
        urls: Arc<Mutex<HashMap<String, Url>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                urls: Arc::new(Mutex::new(HashMap::new())),
            }
        }
    }

    #[async_trait]
    impl Storage for MockStorage {
        async fn create_url(&self, short_id: &str, original_url: &str) -> Result<Url> {
            let url = Url {
                short_id: short_id.to_string(),
                original_url: original_url.to_string(),
                visit_count: 0,
                created_at: UnixNanos::from(Utc::now().timestamp_nanos() as u128),
                last_accessed: None,
            };

            let mut urls = self.urls.lock().unwrap();
            urls.insert(short_id.to_string(), url.clone());

            Ok(url)
        }

        async fn get_url(&self, short_id: &str) -> Result<Url> {
            let urls = self.urls.lock().unwrap();
            urls.get(short_id)
                .cloned()
                .ok_or_else(|| AppError::NotFound(format!("URL with ID {} not found", short_id)))
        }

        async fn increment_visit(&self, short_id: &str) -> Result<()> {
            let mut urls = self.urls.lock().unwrap();
            if let Some(url) = urls.get_mut(short_id) {
                url.visit_count += 1;
                url.last_accessed = Some(UnixNanos::from(Utc::now().timestamp_nanos() as u128));
                Ok(())
            } else {
                Err(AppError::NotFound(format!(
                    "URL with ID {} not found",
                    short_id
                )))
            }
        }

        async fn url_exists(&self, short_id: &str) -> Result<bool> {
            let urls = self.urls.lock().unwrap();
            Ok(urls.contains_key(short_id))
        }

        async fn list_urls(&self, limit: usize, offset: usize) -> Result<Vec<Url>> {
            let urls = self.urls.lock().unwrap();
            let mut all_urls: Vec<_> = urls.values().cloned().collect();

            // Now that we've implemented Ord for UnixNanos, this will work
            all_urls.sort_by(|a, b| b.created_at.cmp(&a.created_at));

            Ok(all_urls.into_iter().skip(offset).take(limit).collect())
        }
    }

    #[tokio::test]
    async fn test_shorten_url() {
        // Setup
        let config = Config {
            database_url: "memory".to_string(),
            server_host: "localhost".to_string(),
            server_port: 8080,
            short_url_length: 6,
            storage_type: crate::storage::StorageType::Sqlite,
            enable_metrics: false,
        };

        let storage = Box::new(MockStorage::new());
        let service = UrlService::new(storage, config);

        // Test
        let url = "https://example.com";
        let result = service.shorten_url(url, None).await.unwrap();

        // Verify
        assert_eq!(result.original_url, url);
        assert_eq!(result.short_id.len(), 6);
    }

    #[tokio::test]
    async fn test_get_url() {
        // Setup
        let config = Config {
            database_url: "memory".to_string(),
            server_host: "localhost".to_string(),
            server_port: 8080,
            short_url_length: 6,
            storage_type: crate::storage::StorageType::Sqlite,
            enable_metrics: false,
        };

        let storage_mock = MockStorage::new();

        // Create a URL first
        let short_id = "testid";
        let original_url = "https://example.com";
        storage_mock
            .create_url(short_id, original_url)
            .await
            .unwrap();

        // Create the service with a new Box
        let storage = Box::new(storage_mock);
        let service = UrlService::new(storage, config);

        // Test
        let result = service.get_url(short_id).await.unwrap();

        // Verify
        assert_eq!(result, original_url);
    }

    #[tokio::test]
    async fn test_custom_short_id() {
        // Setup
        let config = Config {
            database_url: "memory".to_string(),
            server_host: "localhost".to_string(),
            server_port: 8080,
            short_url_length: 6,
            storage_type: crate::storage::StorageType::Sqlite,
            enable_metrics: false,
        };

        let storage = Box::new(MockStorage::new());
        let service = UrlService::new(storage, config);

        // Test
        let url = "https://rust-lang.org";
        let custom_id = "rustlang";
        let result = service
            .shorten_url(url, Some(custom_id.to_string()))
            .await
            .unwrap();

        // Verify
        assert_eq!(result.original_url, url);
        assert_eq!(result.short_id, custom_id);
    }
}
