use crate::application::dto::responses::{ErrorResponse, ExpiringUrlsResponse, UrlInfoResponse};
use crate::domain::repositories::UrlRepository;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, Json};
use tracing::{info, warn};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_error() {
        let error = ErrorResponse {
            error: "DATABASE_ERROR".to_string(),
            message: "Internal server error".to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
        };
        assert_eq!(error.status_code, 500);
    }
}
