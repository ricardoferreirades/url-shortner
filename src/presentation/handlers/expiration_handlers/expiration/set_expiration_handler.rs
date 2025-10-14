use crate::application::dto::{
    requests::SetExpirationRequest,
    responses::{ErrorResponse, SuccessResponse},
};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::Path, extract::State, http::StatusCode, Json};
use tracing::{info, warn};

/// Handler for setting URL expiration
#[utoipa::path(
    put,
    path = "/urls/{short_code}/expiration",
    params(
        ("short_code" = String, Path, description = "Short code to set expiration for")
    ),
    request_body = SetExpirationRequest,
    responses(
        (status = 200, description = "Expiration set successfully", body = SuccessResponse),
        (status = 404, description = "URL not found", body = ErrorResponse),
    ),
    tag = "expiration"
)]
pub async fn set_expiration_handler(
    State(app_state): State<ConcreteAppState>,
    Path(short_code_str): Path<String>,
    Json(request): Json<SetExpirationRequest>,
) -> Result<(StatusCode, Json<SuccessResponse>), (StatusCode, Json<ErrorResponse>)> {
    info!("Setting expiration for short code: {}", short_code_str);

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
            url.expiration_date = Some(request.expiration_date);

            match app_state.url_repository.update_url(&url).await {
                Ok(_) => {
                    info!("Expiration set successfully for URL: {}", url.short_code);
                    let response = SuccessResponse {
                        message: "Expiration set successfully".to_string(),
                        status_code: StatusCode::OK.as_u16(),
                    };
                    Ok((StatusCode::OK, Json(response)))
                }
                Err(error) => {
                    warn!("Failed to update URL expiration: {}", error);
                    let error_response = ErrorResponse {
                        error: "UPDATE_FAILED".to_string(),
                        message: "Failed to update expiration".to_string(),
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
