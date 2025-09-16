use crate::application::dto::{requests::ShortenUrlRequest, responses::ShortenUrlResponse, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use super::app_state::AppState;

/// Handler for shortening URLs
#[utoipa::path(
    post,
    path = "/shorten",
    request_body = ShortenUrlRequest,
    responses(
        (status = 201, description = "URL shortened successfully", body = ShortenUrlResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
    ),
    tag = "url-shortener"
)]
pub async fn shorten_url_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    headers: HeaderMap,
    Json(request): Json<ShortenUrlRequest>,
) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
{
    // Require Authorization: Bearer <token>
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok());
    let token = match auth_header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) if !t.is_empty() => t,
        _ => {
            let error_response = ErrorResponse {
                error: "UNAUTHORIZED".to_string(),
                message: "Missing or invalid Authorization header".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    // Verify token and get user
    let user = match app_state.auth_service.verify_token(token).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Token verification failed: {}", e);
            let error_response = ErrorResponse {
                error: "INVALID_TOKEN".to_string(),
                message: "Invalid or expired token".to_string(),
                status_code: StatusCode::UNAUTHORIZED.as_u16(),
            };
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    let user_id = Some(user.id);
    info!("Received shorten URL request for: {} (user: {:?})", request.url, user_id);

    match app_state.shorten_url_use_case.execute(request, user_id).await {
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
pub async fn redirect_handler<R, U>(
    State(app_state): State<AppState<R, U>>,
    axum::extract::Path(short_code_str): axum::extract::Path<String>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
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
    match app_state.url_repository.find_by_short_code(&short_code).await {
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
