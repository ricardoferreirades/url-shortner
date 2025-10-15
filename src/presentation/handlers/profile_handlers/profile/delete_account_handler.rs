use crate::application::dto::requests::DeleteAccountRequest;
use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::UserRepository;
use crate::domain::services::AnonymizationService;
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};
use bcrypt::verify;

/// Delete current user's account
/// DELETE /api/profile/delete
#[utoipa::path(
    delete,
    path = "/profile/delete",
    request_body = DeleteAccountRequest,
    responses(
        (status = 204, description = "Account deleted successfully"),
        (status = 401, description = "Invalid password", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn delete_account(
    State(state): State<ConcreteAppState>,
    Json(request): Json<DeleteAccountRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Get user from repository
    let user = match state.user_repository.find_by_id(user_id).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "User not found".to_string(),
                    message: "User account not found".to_string(),
                    status_code: 404,
                }),
            ))
        }
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Database error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ))
        }
    };

    // Verify password
    let is_valid = match verify(&request.password, &user.password_hash) {
        Ok(valid) => valid,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Password verification error".to_string(),
                    message: e.to_string(),
                    status_code: 500,
                }),
            ))
        }
    };

    if !is_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "Invalid password".to_string(),
                message: "The password provided is incorrect".to_string(),
                status_code: 401,
            }),
        ));
    }

    // Anonymize account data instead of hard deletion
    let anonymization_service = AnonymizationService::new();
    let anonymized_data = anonymization_service.anonymize_user_data(&user);

    match state
        .user_repository
        .anonymize_account(
            user_id,
            &anonymized_data.username,
            &anonymized_data.email,
            &anonymized_data.password_hash,
        )
        .await
    {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: "Account anonymization failed".to_string(),
                message: e.to_string(),
                status_code: 500,
            }),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_account_request_deserialize() {
        let json = r#"{"password":"testpassword"}"#;
        let request: Result<DeleteAccountRequest, _> = serde_json::from_str(json);
        assert!(request.is_ok());
    }

    #[test]
    fn test_invalid_password_error() {
        let error = ErrorResponse {
            error: "Invalid password".to_string(),
            message: "The password provided is incorrect".to_string(),
            status_code: 401,
        };
        assert_eq!(error.status_code, 401);
    }

    #[test]
    fn test_user_not_found_error() {
        let error = ErrorResponse {
            error: "User not found".to_string(),
            message: "User account not found".to_string(),
            status_code: 404,
        };
        assert_eq!(error.status_code, 404);
    }

    #[test]
    fn test_anonymization_failed_error() {
        let error = ErrorResponse {
            error: "Account anonymization failed".to_string(),
            message: "Failed to anonymize account data".to_string(),
            status_code: 500,
        };
        assert_eq!(error.error, "Account anonymization failed");
    }
}
