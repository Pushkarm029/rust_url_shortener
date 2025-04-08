// use rust_url_shortener::errors::Result;
// use rust_url_shortener::models::{CreateUrlRequest, UrlResponse, Url};
// use rust_url_shortener::storage::{Storage, StorageType};
// use std::sync::Arc;
// use axum::http::{Method, StatusCode};
// use axum::routing::{get, post};
// use axum::{Json, Router};
// use tower_http::cors::{Any, CorsLayer};
// use axum_test::TestClient;

// async fn setup_test_app() -> Result<Router> {
//     // Initialize storage - use in-memory database for testing
//     let storage = rust_url_shortener::storage::create_storage(
//         &StorageType::Sqlite,
//         ":memory:",
//     ).await?;

//     // Create service
//     let config = rust_url_shortener::config::Config {
//         database_url: ":memory:".to_string(),
//         server_host: "127.0.0.1".to_string(),
//         server_port: 0, // Random port
//         short_url_length: 6,
//         storage_type: StorageType::Sqlite,
//         enable_metrics: false,
//     };

//     let url_service = Arc::new(rust_url_shortener::services::UrlService::new(storage, config));

//     // Configure CORS
//     let cors = CorsLayer::new()
//         .allow_methods([Method::GET, Method::POST])
//         .allow_origin(Any);

//     // Build the router
//     let app = rust_url_shortener::routes::url_routes(url_service)
//         .layer(cors);

//     Ok(app)
// }

// #[tokio::test]
// async fn test_shorten_url() -> Result<()> {
//     // Setup the app
//     let app = setup_test_app().await?;

//     // Create a test client
//     let client = TestClient::new(app);

//     // Test shortening a URL
//     let response = client
//         .post("/api/shorten")
//         .json(&CreateUrlRequest {
//             url: "https://example.com".to_string(),
//             custom_id: None,
//         })
//         .await;

//     // Assert status code
//     assert_eq!(response.status(), StatusCode::CREATED);

//     // Parse response
//     let url_response = response.json::<UrlResponse>().await;

//     // Verify the response
//     assert_eq!(url_response.original_url, "https://example.com");
//     assert_eq!(url_response.short_id.len(), 6);

//     Ok(())
// }

// #[tokio::test]
// async fn test_custom_short_id() -> Result<()> {
//     // Setup the app
//     let app = setup_test_app().await?;

//     // Create a test client
//     let client = TestClient::new(app);

//     // Test shortening a URL with a custom ID
//     let response = client
//         .post("/api/shorten")
//         .json(&CreateUrlRequest {
//             url: "https://rust-lang.org".to_string(),
//             custom_id: Some("rustlang".to_string()),
//         })
//         .await;

//     // Assert status code
//     assert_eq!(response.status(), StatusCode::CREATED);

//     // Parse response
//     let url_response = response.json::<UrlResponse>().await;

//     // Verify the response
//     assert_eq!(url_response.original_url, "https://rust-lang.org");
//     assert_eq!(url_response.short_id, "rustlang");

//     Ok(())
// }

// #[tokio::test]
// async fn test_get_url_stats() -> Result<()> {
//     // Setup the app
//     let app = setup_test_app().await?;

//     // Create a test client
//     let client = TestClient::new(app);

//     // First create a URL
//     let create_response = client
//         .post("/api/shorten")
//         .json(&CreateUrlRequest {
//             url: "https://example.com".to_string(),
//             custom_id: Some("test123".to_string()),
//         })
//         .await;

//     assert_eq!(create_response.status(), StatusCode::CREATED);

//     // Now get stats for the URL
//     let stats_response = client
//         .get("/api/stats/test123")
//         .await;

//     // Assert success
//     assert_eq!(stats_response.status(), StatusCode::OK);

//     // Parse response
//     let stats = stats_response.json::<Url>().await;

//     // Verify stats
//     assert_eq!(stats.original_url, "https://example.com");
//     assert_eq!(stats.short_id, "test123");
//     assert_eq!(stats.visit_count, 0); // No visits yet

//     Ok(())
// }
