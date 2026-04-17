use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub enum ServiceError {
    NotFound(String),
    InvalidArgument(String),
    Internal(String),
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            ServiceError::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            ServiceError::InvalidArgument(msg) => (StatusCode::BAD_REQUEST, "invalid_argument", msg),
            ServiceError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", msg),
        };
        let body = json!({ "code": code, "message": message });
        (status, axum::Json(body)).into_response()
    }
}
