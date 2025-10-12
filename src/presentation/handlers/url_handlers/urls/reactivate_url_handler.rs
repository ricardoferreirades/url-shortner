use crate::application::dto::ErrorResponse;
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::app_state::AppState;
use axum::{
    extract::State,
    http::HeaderMap,
    http::{header, StatusCode},
    Json,
};
use tracing::{info, warn};

/// Handler for reactivating a URL
#[utoipa::path(
    patch,
    path = "/urls/{id}/reactivate",
    params(
        ("id" = i32, Path, description = "URL ID to reactivate")
    ),
    responses(
        (status = 204, description = "URL reactivated successfully"),
        (status = 404, description = "URL not found", body = ErrorResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
    ),
    tag = "url-management"
)]
pub async fn reactivate_url_handler<R, U, P>(
    State(app_state): State<AppState<R, U, P>>,
    headers: HeaderMap,
    axum::extract::Path(id): axum::extract::Path<i32>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)>
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

    info!(
        "Received reactivate URL request for ID: {} (user: {})",
        id, user.id
    );

    match app_state
        .url_service
        .reactivate_url(id, Some(user.id))
        .await
    {
        Ok(true) => {
            info!("Successfully reactivated URL ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Ok(false) => {
            warn!("URL not found or not owned by user: {}", id);
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "URL not found or you don't have permission to reactivate it".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Failed to reactivate URL {}: {}", id, error);
            let error_response = ErrorResponse {
                error: "REACTIVATE_FAILED".to_string(),
                message: error.to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}
