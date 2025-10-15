use crate::application::dto::responses::{ErrorResponse, ExpirationInfoResponse};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for getting expiration information for a URL
#[utoipa::path(
    get,
    path = "/urls/{short_code}/expiration",
    params(
        ("short_code" = String, Path, description = "Short code to check expiration for")
    ),
    responses(
        (status = 200, description = "Expiration information retrieved", body = ExpirationInfoResponse),
        (status = 404, description = "URL not found", body = ErrorResponse),
    ),
    tag = "expiration"
)]
pub async fn get_expiration_info_handler(
    State(app_state): State<ConcreteAppState>,
    Path(short_code_str): Path<String>,
) -> Result<(StatusCode, Json<ExpirationInfoResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("Getting expiration info for short code: {}", short_code_str);

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
        Ok(Some(url)) => {
            let now = chrono::Utc::now();
            let expires_in_days = url.expiration_date.map(|exp| {
                let duration = exp - now;
                duration.num_days()
            });

            let response = ExpirationInfoResponse {
                expiration_date: url.expiration_date.map(|d| d.to_rfc3339()),
                is_expired: url.is_expired(),
                expires_in_days,
            };

            Ok((StatusCode::OK, Json(response)))
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
    fn test_not_found_error() {
        let error = ErrorResponse {
            error: "NOT_FOUND".to_string(),
            message: "URL not found".to_string(),
            status_code: StatusCode::NOT_FOUND.as_u16(),
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_invalid_short_code_error() {
        let error = ErrorResponse {
            error: "INVALID_SHORT_CODE".to_string(),
            message: "Short code is invalid".to_string(),
            status_code: StatusCode::BAD_REQUEST.as_u16(),
        };
        assert_eq!(error.error, "INVALID_SHORT_CODE");
    }
}
