use crate::application::dto::{requests::{ShortenUrlRequest, BulkShortenUrlsRequest, BatchUrlOperationRequest, BulkStatusUpdateRequest, BulkExpirationUpdateRequest, BulkDeleteRequest}, responses::{ShortenUrlResponse, BatchOperationResponse, BatchOperationResult, BulkOperationProgress}, ErrorResponse};
use crate::domain::repositories::UrlRepository;
use axum::{extract::State, http::{StatusCode, header}, response::Redirect, Json, http::HeaderMap};
use tracing::{info, warn};
use crate::presentation::handlers::app_state::AppState;

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
pub async fn shorten_url_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    headers: HeaderMap,
    Json(request): Json<ShortenUrlRequest>,
) -> Result<(StatusCode, Json<ShortenUrlResponse>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,{
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
