use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use serde::Deserialize;

use crate::{errors::Result, models::CreateUrlRequest, services::UrlService};

pub fn url_routes(url_service: Arc<UrlService>) -> Router {
    Router::new()
        .route("/api/shorten", post(shorten_url))
        .route("/api/urls", get(list_urls))
        .route("/api/stats/{short_id}", get(get_url_stats))
        .route("/{short_id}", get(redirect_url))
        .with_state(url_service)
}

/// Handler for shortening URLs
async fn shorten_url(
    State(url_service): State<Arc<UrlService>>,
    Json(payload): Json<CreateUrlRequest>,
) -> Result<impl IntoResponse> {
    let response = url_service
        .shorten_url(&payload.url, payload.custom_id)
        .await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Handler for redirecting to the original URL
async fn redirect_url(
    State(url_service): State<Arc<UrlService>>,
    Path(short_id): Path<String>,
) -> Result<impl IntoResponse> {
    let original_url = url_service.get_url(&short_id).await?;
    Ok(Redirect::permanent(&original_url))
}

/// Handler for getting URL statistics
async fn get_url_stats(
    State(url_service): State<Arc<UrlService>>,
    Path(short_id): Path<String>,
) -> Result<impl IntoResponse> {
    let stats = url_service.get_url_stats(&short_id).await?;
    Ok(Json(stats))
}

/// Query parameters for listing URLs
#[derive(Debug, Deserialize)]
struct ListUrlsQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

/// Handler for listing all URLs
async fn list_urls(
    State(url_service): State<Arc<UrlService>>,
    Query(params): Query<ListUrlsQuery>,
) -> Result<impl IntoResponse> {
    let limit = params.limit.unwrap_or(10);
    let offset = params.offset.unwrap_or(0);

    let urls = url_service.list_urls(limit, offset).await?;
    Ok(Json(urls))
}
