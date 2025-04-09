use serde::{Deserialize, Serialize};

use super::UnixNanos;

// URL metadata including original URL and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Url {
    pub short_id: String,
    pub original_url: String,
    pub visit_count: u32,
    pub created_at: UnixNanos,
    pub last_accessed: Option<UnixNanos>,
}

// Request payload for creating a shortened URL
#[derive(Debug, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub custom_id: Option<String>, // Optional custom alias
}

// Response payload for a shortened URL
#[derive(Debug, Serialize)]
pub struct UrlResponse {
    pub short_id: String,
    pub original_url: String,
}
