use crate::application::dto::{requests::{ShortenUrlRequest, BulkShortenUrlsRequest, BatchUrlOperationRequest, BulkStatusUpdateRequest, BulkExpirationUpdateRequest, BulkDeleteRequest}, responses::{ShortenUrlResponse, BatchOperationResponse, BatchOperationResult, BulkOperationProgress}, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use crate::presentation::handlers::app_state::AppState;

/// Handler for redirecting to original URL
#[utoipa::path(
    get,
    path = "/{short_code}",
    params(
        ("short_code" = String, Path, description = "Short code to redirect")
    ),
    responses(
        (status = 301, description = "Redirect to original URL"),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 404, description = "Short code not found", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn redirect_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    axum::extract::Path(short_code_str): axum::extract::Path<String>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,{
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

    // Find the URL with validation (checks expiration and status)
    match app_state.url_service.get_url_by_short_code_with_validation(&short_code).await {
        Ok(Some(url)) => {
            info!("Redirecting {} to {}", short_code.value(), url.original_url);
            Ok(Redirect::permanent(&url.original_url))
        }
        Ok(None) => {
            warn!("Short code not found or not accessible: {}", short_code.value());
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "Short code not found or no longer available".to_string(),
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
