use axum::{
    Json,
    extract::State,
    response::Redirect,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use seahash;
use tracing::info;
use crate::database::Database;

#[derive(Deserialize, ToSchema)]
pub struct ShortenUrlRequest {
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct ShortenUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

#[utoipa::path(
    post,
    path = "/shorten",
    request_body = ShortenUrlRequest,
    responses(
        (status = 200, description = "Shortened URL created", body = ShortenUrlResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn shorten_url_handler(
    State(db): State<Database>,
    Json(payload): Json<ShortenUrlRequest>,
) -> Result<Json<ShortenUrlResponse>, StatusCode> {
    // Generate a short hash from the URL
    let hash = generate_short_code(&payload.url);
    
    // Try to save to database
    match db.create_url(&hash, &payload.url).await {
        Ok(_) => {
            let short_url = format!("http://localhost:8000/{}", hash);
            info!("Shortened URL: {} -> {}", payload.url, short_url);
            
            Ok(Json(ShortenUrlResponse {
                short_url,
                original_url: payload.url,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to save URL to database: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[utoipa::path(
    get,
    path = "/{short_code}",
    params(
        ("short_code" = String, Path, description = "Short code to redirect"),
    ),
    responses(
        (status = 308, description = "Permanent redirect to original URL"),
        (status = 404, description = "Short code not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn redirect_handler(
    State(db): State<Database>,
    axum::extract::Path(short_code): axum::extract::Path<String>,
) -> Result<Redirect, StatusCode> {
    match db.get_url_by_short_code(&short_code).await {
        Ok(Some(url_record)) => {
            info!("Redirecting {} to {}", short_code, url_record.original_url);
            Ok(Redirect::permanent(&url_record.original_url))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn generate_short_code(url: &str) -> String {
    // Hash the URL using seahash
    let hash = seahash::hash(url.as_bytes());
    // Format the hash as hexadecimal and take the first 8 characters as the short code
    format!("{:x}", hash)[..8].to_string()
}
