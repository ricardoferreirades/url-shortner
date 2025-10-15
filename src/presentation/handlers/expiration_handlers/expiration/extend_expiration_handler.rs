use crate::application::dto::{
    requests::ExtendExpirationRequest,
    responses::{ErrorResponse, SuccessResponse},
};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for extending URL expiration
#[utoipa::path(
    post,
    path = "/urls/{short_code}/extend",
    params(
        ("short_code" = String, Path, description = "Short code to extend expiration for")
    ),
    request_body = ExtendExpirationRequest,
    responses(
        (status = 200, description = "Expiration extended successfully", body = SuccessResponse),
        (status = 404, description = "URL not found", body = ErrorResponse),
    ),
    tag = "expiration"
)]
pub async fn extend_expiration_handler(
    State(app_state): State<ConcreteAppState>,
    Path(short_code_str): Path<String>,
    Json(request): Json<ExtendExpirationRequest>,
) -> Result<(StatusCode, Json<SuccessResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("Extending expiration for short code: {}", short_code_str);

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

    match app_state
        .url_repository
        .find_by_short_code(&short_code)
        .await
    {
        Ok(Some(mut url)) => {
            let now = chrono::Utc::now();
            let current_expiration = url.expiration_date.unwrap_or(now);
            let new_expiration =
                current_expiration + chrono::Duration::days(request.additional_days as i64);
            url.expiration_date = Some(new_expiration);

            match app_state.url_repository.update_url(&url).await {
                Ok(_) => {
                    info!(
                        "Expiration extended successfully for URL: {}",
                        url.short_code
                    );
                    let response = SuccessResponse {
                        message: format!("Expiration extended by {} days", request.additional_days),
                        status_code: StatusCode::OK.as_u16(),
                    };
                    Ok((StatusCode::OK, Json(response)))
                }
                Err(error) => {
                    warn!("Failed to extend URL expiration: {}", error);
                    let error_response = ErrorResponse {
                        error: "UPDATE_FAILED".to_string(),
                        message: "Failed to extend expiration".to_string(),
                        status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                    };
                    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
                }
            }
        }
        Ok(None) => {
            warn!("URL not found: {}", short_code.value());
            let error_response = ErrorResponse {
                error: "NOT_FOUND".to_string(),
                message: "URL not found".to_string(),
                status_code: StatusCode::NOT_FOUND.as_u16(),
            };
            Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        Err(error) => {
            warn!("Database error: {}", error);
            let error_response = ErrorResponse {
                error: "DATABASE_ERROR".to_string(),
                message: "Internal server error".to_string(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            };
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_failed_error() {
        let error = ErrorResponse {
            error: "UPDATE_FAILED".to_string(),
            message: "Failed to extend expiration".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.error, "UPDATE_FAILED");
    }

    #[test]
    fn test_not_found_error() {
        let error = ErrorResponse {
            error: "NOT_FOUND".to_string(),
            message: "URL not found".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }
}
