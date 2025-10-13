use crate::application::dto::responses::ErrorResponse;
use crate::domain::repositories::UserRepository;
use crate::domain::services::{FileUploadError, FileUploadService};
use crate::presentation::handlers::ConcreteAppState;
use axum::{extract::State, http::StatusCode, response::Json};
use axum_extra::extract::Multipart;
use serde_json::Value;

/// Upload profile picture
/// POST /api/profile/avatar
#[utoipa::path(
    post,
    path = "/profile/avatar",
    request_body = String,
    responses(
        (status = 200, description = "Avatar uploaded successfully", body = serde_json::Value),
        (status = 400, description = "Invalid file", body = ErrorResponse),
        (status = 413, description = "File too large", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "profile"
)]
pub async fn upload_profile_picture(
    State(state): State<ConcreteAppState>,
    mut multipart: Multipart,
    // In a real implementation, you would extract user from JWT token
    // For now, we'll use a placeholder user_id
) -> Result<Json<Value>, (StatusCode, Json<ErrorResponse>)> {
    // TODO: Extract user_id from JWT token
    let user_id = 1; // Placeholder

    // Create file upload service
    let upload_service =
        FileUploadService::new_profile_picture_service("uploads/avatars".to_string());

    // Process multipart form
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Multipart error".to_string(),
                message: e.to_string(),
                status_code: 400,
            }),
        )
    })? {
        if field.name() == Some("avatar") {
            let filename = field
                .file_name()
                .ok_or_else(|| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ErrorResponse {
                            error: "Invalid file".to_string(),
                            message: "No filename provided".to_string(),
                            status_code: 400,
                        }),
                    )
                })?
                .to_string();

            let content_type = field
                .content_type()
                .unwrap_or("application/octet-stream")
                .to_string();

            let data = field.bytes().await.map_err(|e| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "File read error".to_string(),
                        message: e.to_string(),
                        status_code: 400,
                    }),
                )
            })?;

            // Process and save file
            let upload_result = upload_service
                .process_and_save_file(&filename, &content_type, data.to_vec())
                .await
                .map_err(|e| {
                    let (status, message) = match e {
                        FileUploadError::FileTooLarge(_, max) => (
                            StatusCode::PAYLOAD_TOO_LARGE,
                            format!("File too large. Maximum size: {} bytes", max),
                        ),
                        FileUploadError::InvalidFileType(_, allowed) => (
                            StatusCode::BAD_REQUEST,
                            format!("Invalid file type. Allowed types: {:?}", allowed),
                        ),
                        _ => (StatusCode::BAD_REQUEST, e.to_string()),
                    };
                    (
                        status,
                        Json(ErrorResponse {
                            error: "File upload error".to_string(),
                            message,
                            status_code: status.as_u16(),
                        }),
                    )
                })?;

            // Update user's avatar URL
            let avatar_url =
                upload_service.get_file_url(&upload_result.filename, "http://localhost:8000");

            match state
                .user_repository
                .update_profile(
                    user_id,
                    None,              // first_name
                    None,              // last_name
                    None,              // bio
                    Some(&avatar_url), // avatar_url
                    None,              // website
                    None,              // location
                    None,              // privacy
                )
                .await
            {
                Ok(_) => {
                    return Ok(Json(serde_json::json!({
                        "message": "Avatar uploaded successfully",
                        "avatar_url": avatar_url,
                        "filename": upload_result.filename,
                        "file_size": upload_result.file_size,
                        "width": upload_result.width,
                        "height": upload_result.height
                    })));
                }
                Err(e) => {
                    // Clean up uploaded file on database error
                    let _ = upload_service.delete_file(&upload_result.filename).await;
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: "Database error".to_string(),
                            message: e.to_string(),
                            status_code: 500,
                        }),
                    ));
                }
            }
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: "No file provided".to_string(),
            message: "No avatar file found in request".to_string(),
            status_code: 400,
        }),
    ))
}
