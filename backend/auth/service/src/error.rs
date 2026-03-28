//! JSON API errors returned to clients (no stack traces).

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Stable error envelope for HTTP JSON responses.
#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<&'static str>,
}

/// Application-level HTTP errors for auth routes.
#[derive(Debug)]
pub enum ApiError {
    BadRequest(&'static str),
    Conflict(&'static str),
    Unauthorized,
    Internal,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg, code) = match self {
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m, Some("bad_request")),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m, Some("conflict")),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", Some("unauthorized")),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "internal error", None),
        };
        let body = ErrorBody {
            error: msg.to_string(),
            code,
        };
        (status, Json(body)).into_response()
    }
}

impl From<sea_orm::DbErr> for ApiError {
    fn from(e: sea_orm::DbErr) -> Self {
        tracing::error!(?e, "database error");
        ApiError::Internal
    }
}
