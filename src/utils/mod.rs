use rand::{Rng, distributions::Alphanumeric};

/// Generate a random string of specified length for short IDs
pub fn generate_short_id(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Validate a URL string
pub fn validate_url(url: &str) -> bool {
    // Basic URL validation - in a real application, use a proper URL validation library
    url.starts_with("http://") || url.starts_with("https://")
}

/// Validate a short ID (only alphanumeric characters allowed)
pub fn validate_short_id(short_id: &str) -> bool {
    short_id.chars().all(|c| c.is_alphanumeric())
}
