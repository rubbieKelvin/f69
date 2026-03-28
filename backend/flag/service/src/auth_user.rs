//! Extractor: validates `Authorization: Bearer` using JWKS from the auth service.

use std::future::Future;

use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header, request::Parts, StatusCode},
    Json,
};
use serde_json::json;
use uuid::Uuid;

use crate::state::AppState;

/// Authenticated subject resolved from a valid JWT (`sub` claim = user id).
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = (StatusCode, Json<serde_json::Value>);

    fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let state = AppState::from_ref(state);
        // Clone header value here: `Parts` cannot be held across the async block.
        let auth = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        async move {
            let auth = auth.ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "missing authorization", "code": "unauthorized" })),
                )
            })?;

            let token = auth.strip_prefix("Bearer ").ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "invalid authorization scheme", "code": "unauthorized" })),
                )
            })?;

            let jwks = state.jwks.get().await.map_err(|e| {
                tracing::error!(?e, "jwks fetch failed");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({ "error": "auth service unavailable", "code": "jwks_unavailable" })),
                )
            })?;

            let claims = state.jwt.validate(token, &jwks).map_err(|e| {
                tracing::debug!(?e, "jwt validation failed");
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "invalid token", "code": "unauthorized" })),
                )
            })?;

            let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "invalid subject", "code": "unauthorized" })),
                )
            })?;

            Ok(AuthUser { user_id })
        }
    }
}
