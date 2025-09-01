use axum::{
    Json,
};
use serde::{Deserialize, Serialize};
use seahash;
use tracing::info;

#[derive(Deserialize)]
pub struct ShortenUrlRequest {
    pub url: String,
}

#[derive(Serialize)]
pub struct ShortenUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

pub async fn shorten_url_handler(Json(payload): Json<ShortenUrlRequest>) -> Json<ShortenUrlResponse> {
    // Generate a short hash from the URL
    let hash = generate_short_code(&payload.url);
    
    // Create the short URL
    let short_url = format!("http://localhost:8000/{}", hash);
    
    info!("Shortened URL: {} -> {}", payload.url, short_url);
    
    Json(ShortenUrlResponse {
        short_url,
        original_url: payload.url,
    })
}

pub fn generate_short_code(url: &str) -> String {
    // Hash the URL using seahash
    let hash = seahash::hash(url.as_bytes());
    // Format the hash as hexadecimal and take the first 8 characters as the short code
    format!("{:x}", hash)[..8].to_string()
}
