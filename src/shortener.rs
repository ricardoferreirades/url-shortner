use axum::{
    Json,
    extract::State,
    response::Redirect,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use seahash;
use tracing::{info, warn};
use crate::database::Database;
use crate::validation::{validate_url, validate_short_code, ValidationConfig};

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ShortenUrlRequest {
    pub url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ShortenUrlResponse {
    pub short_url: String,
    pub original_url: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

#[utoipa::path(
    post,
    path = "/shorten",
    request_body = ShortenUrlRequest,
    responses(
        (status = 200, description = "Shortened URL created", body = ShortenUrlResponse),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn shorten_url_handler(
    State(db): State<Database>,
    Json(payload): Json<ShortenUrlRequest>,
) -> Result<Json<ShortenUrlResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validate the URL
    let config = ValidationConfig::default();
    let validated_url = match validate_url(&payload.url, &config) {
        Ok(url) => url,
        Err(validation_error) => {
            warn!("URL validation failed: {}", validation_error);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "validation_error".to_string(),
                    message: validation_error.to_string(),
                    status_code: 400,
                })
            ));
        }
    };

    // Generate a short hash from the validated URL
    let hash = generate_short_code(&validated_url);
    
    // Try to save to database
    match db.create_url(&hash, &validated_url).await {
        Ok(_) => {
            let short_url = format!("http://localhost:8000/{}", hash);
            info!("Shortened URL: {} -> {}", validated_url, short_url);
            
            Ok(Json(ShortenUrlResponse {
                short_url,
                original_url: validated_url,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to save URL to database: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "database_error".to_string(),
                    message: "Failed to save URL to database".to_string(),
                    status_code: 500,
                })
            ))
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
        (status = 400, description = "Invalid short code", body = ErrorResponse),
        (status = 404, description = "Short code not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn redirect_handler(
    State(db): State<Database>,
    axum::extract::Path(short_code): axum::extract::Path<String>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    // Validate the short code
    let validated_short_code = match validate_short_code(&short_code) {
        Ok(code) => code,
        Err(validation_error) => {
            warn!("Short code validation failed: {}", validation_error);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "validation_error".to_string(),
                    message: validation_error.to_string(),
                    status_code: 400,
                })
            ));
        }
    };

    match db.get_url_by_short_code(&validated_short_code).await {
        Ok(Some(url_record)) => {
            info!("Redirecting {} to {}", validated_short_code, url_record.original_url);
            Ok(Redirect::permanent(&url_record.original_url))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: format!("Short code '{}' not found", validated_short_code),
                status_code: 404,
            })
        )),
        Err(e) => {
            tracing::error!("Database error: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "database_error".to_string(),
                    message: "Database error occurred".to_string(),
                    status_code: 500,
                })
            ))
        }
    }
}

pub fn generate_short_code(url: &str) -> String {
    // Hash the URL using seahash
    let hash = seahash::hash(url.as_bytes());
    // Format the hash as hexadecimal and take the first 8 characters as the short code
    format!("{:x}", hash)[..8].to_string()
}

