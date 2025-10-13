use crate::application::dto::{
    requests::{ExtendExpirationRequest, SetExpirationRequest},
    responses::{
        ErrorResponse, ExpirationInfoResponse, ExpiringUrlsResponse, SuccessResponse,
        UrlInfoResponse,
    },
};
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

/// Handler for getting URLs expiring soon
#[utoipa::path(
    get,
    path = "/urls/expiring-soon",
    params(
        ("days" = u32, Query, description = "Number of days to look ahead for expiring URLs")
    ),
    responses(
        (status = 200, description = "Expiring URLs retrieved", body = ExpiringUrlsResponse),
    ),
    tag = "expiration"
)]
pub async fn get_expiring_urls_handler(
    State(app_state): State<ConcreteAppState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, u32>>,
) -> Result<(StatusCode, Json<ExpiringUrlsResponse>), (StatusCode, Json<ErrorResponse>)> {
    let days = params.get("days").copied().unwrap_or(7); // Default to 7 days
    info!("Getting URLs expiring within {} days", days);

    let duration = chrono::Duration::days(days as i64);

    match app_state
        .url_repository
        .find_urls_expiring_soon(duration)
        .await
    {
        Ok(urls) => {
            let total_count = urls.len() as i64;
            let url_responses: Vec<UrlInfoResponse> = urls
                .into_iter()
                .map(|url| {
                    UrlInfoResponse {
                        id: url.id,
                        short_code: url.short_code.clone(),
                        original_url: url.original_url.clone(),
                        short_url: url.short_url("https://short.ly"), // TODO: Get from config
                        created_at: url.created_at.to_rfc3339(),
                        expiration_date: url.expiration_date.map(|d| d.to_rfc3339()),
                        is_expired: url.is_expired(),
                        click_count: None, // TODO: Add click tracking
                    }
                })
                .collect();

            let response = ExpiringUrlsResponse {
                urls: url_responses,
                total_count,
                warning_period_days: days,
            };

            Ok((StatusCode::OK, Json(response)))
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
