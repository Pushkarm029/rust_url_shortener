use std::sync::Arc;

use axum::http::Method;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

mod config;
mod errors;
mod models;
mod routes;
mod services;
mod storage;
mod utils;

use config::Config;
use services::UrlService;
use storage::create_storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Initialize storage
    let storage = create_storage(&config.storage_type, &config.database_url).await?;

    // Initialize service
    let url_service = Arc::new(UrlService::new(storage, config.clone()));

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    // Configure tracing
    let trace_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    // Initialize metrics if enabled
    #[cfg(feature = "metrics")]
    {
        tracing::info!("Metrics enabled");
        // In a real application, initialize Prometheus metrics here
    }

    // Build the router
    let app = routes::url_routes(url_service)
        .layer(cors)
        .layer(trace_layer);

    // Start the server
    let addr = format!("{}:{}", config.server_host, config.server_port);
    tracing::info!("Starting server at {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
