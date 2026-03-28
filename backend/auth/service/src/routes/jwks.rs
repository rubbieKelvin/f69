//! JWKS document for JWT verification in peer services.

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::state::AppState;

/// Public keys for validating RS256 tokens issued by this service.
pub async fn jwks(State(state): State<AppState>) -> impl IntoResponse {
    match state.jwt.jwks_json(&state.jwt_private_pem) {
        Ok(json) => (StatusCode::OK, [("content-type", "application/json")], json).into_response(),
        Err(e) => {
            tracing::error!(?e, "jwks build failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
