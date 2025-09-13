use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Custom error response for middleware
#[derive(Debug)]
pub struct MiddlewareError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for MiddlewareError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "error": self.status.as_str(),
            "message": self.message,
            "status_code": self.status.as_u16()
        }));

        (self.status, body).into_response()
    }
}

/// Handle JSON parsing errors
pub fn handle_json_error(err: JsonRejection) -> MiddlewareError {
    match err {
        JsonRejection::JsonDataError(err) => MiddlewareError {
            status: StatusCode::BAD_REQUEST,
            message: format!("Invalid JSON: {}", err),
        },
        JsonRejection::JsonSyntaxError(err) => MiddlewareError {
            status: StatusCode::BAD_REQUEST,
            message: format!("JSON syntax error: {}", err),
        },
        JsonRejection::MissingJsonContentType(err) => MiddlewareError {
            status: StatusCode::UNSUPPORTED_MEDIA_TYPE,
            message: format!("Missing Content-Type: {}", err),
        },
        JsonRejection::BytesRejection(err) => MiddlewareError {
            status: StatusCode::BAD_REQUEST,
            message: format!("Failed to read request body: {}", err),
        },
        _ => MiddlewareError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: "Unknown JSON error".to_string(),
        },
    }
}
