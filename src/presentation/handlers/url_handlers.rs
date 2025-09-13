use crate::application::dto::{requests::ShortenUrlRequest, responses::ShortenUrlResponse, ErrorResponse};
use crate::application::use_cases::ShortenUrlUseCase;
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::StatusCode, response::Redirect, Json};
use tracing::{info, warn};

/// Handler for shortening URLs
pub async fn shorten_url_handler<R>(
    State(use_case): State<ShortenUrlUseCase<R>>,
    Json(request): Json<ShortenUrlRequest>,
) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync,
{
    info!("Received shorten URL request for: {}", request.url);

    match use_case.execute(request, None).await {
        Ok(response) => {
            info!("Successfully shortened URL: {} -> {}", response.original_url, response.short_url);
            Ok((StatusCode::CREATED, Json(response)))
        }
        Err(error) => {
            warn!("Failed to shorten URL: {}", error);
            let error_response = ErrorResponse {
                error: "SHORTEN_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            Err((StatusCode::BAD_REQUEST, Json(error_response)))
        }
    }
}

/// Handler for redirecting to original URL
pub async fn redirect_handler<R>(
    State(repository): State<R>,
    axum::extract::Path(short_code_str): axum::extract::Path<String>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync,
{
    info!("Received redirect request for short code: {}", short_code_str);

    // Parse and validate short code
    let short_code = match crate::domain::entities::ShortCode::new(short_code_str) {
        Ok(code) => code,
        Err(error) => {
            warn!("Invalid short code format: {}", error);
            let error_response = ErrorResponse {
                error: "INVALID_SHORT_CODE".to_string(),
                message: error.to_string(),
                status_code: StatusCode::BAD_REQUEST.as_u16(),
            };
            return Err((StatusCode::BAD_REQUEST, Json(error_response)));
        }
    };

    // Find the URL
    match repository.find_by_short_code(&short_code).await {
        Ok(Some(url)) => {
            info!("Redirecting {} to {}", short_code.value(), url.original_url);
            Ok(Redirect::permanent(&url.original_url))
        }
        Ok(None) => {
            warn!("Short code not found: {}", short_code.value());
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "Short code not found".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Database error while looking up short code: {}", error);
            let error_response = ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Internal server error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
