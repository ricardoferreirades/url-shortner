use crate::application::dto::{
    requests::{BulkShortenUrlsRequest, ShortenUrlRequest},
    responses::ShortenUrlResponse,
    ErrorResponse,
};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::app_state::AppState;
use axum::{
    extract::State,
    http::HeaderMap,
    http::{header, StatusCode},
    Json,
};
use tracing::warn;

/// Handler for bulk shortening URLs
#[utoipa::path(
    post,
    path = "/urls/bulk",
    request_body = BulkShortenUrlsRequest,
    responses(
        (status = 201, description = "URLs shortened successfully", body = [ShortenUrlResponse]),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "bulk-operations"
)]
pub async fn bulk_shorten_urls_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    headers: HeaderMap,
    Json(request): Json<BulkShortenUrlsRequest>,
) -> Result<(StatusCode, Json<Vec<ShortenUrlResponse>>), (StatusCode, Json<ErrorResponse>)>
where
    R: UrlRepository + Send + Sync + Clone,
    U: crate::domain::repositories::UserRepository + Send + Sync + Clone,
    P: crate::domain::repositories::PasswordResetRepository + Send + Sync + Clone,
{
    // Require Authorization: Bearer <token>
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
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
    let mut responses: Vec<ShortenUrlResponse> = Vec::with_capacity(request.items.len());

    for item in request.items {
        let req = ShortenUrlRequest {
            url: item.url,
            custom_short_code: item.custom_short_code,
            expiration_date: item.expiration_date,
        };
        match app_state.shorten_url_use_case.execute(req, user_id).await {
            Ok(resp) => responses.push(resp),
            Err(err) => {
                let error_response = ErrorResponse {
                    error: "SHORTEN_FAILED".to_string(),
                    message: err.to_string(),
                    status_code: StatusCode::BAD_REQUEST.as_u16(),
                };
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }
        }
    }

    Ok((StatusCode::CREATED, Json(responses)))
}
