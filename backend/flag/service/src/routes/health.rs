use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthResponse { status: "ok" }),
    )
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
}

pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.ping().await {
        Ok(()) => (
            StatusCode::OK,
            Json(ReadyResponse { status: "ready" }),
        )
            .into_response(),
        _ => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "database unavailable" })),
        )
            .into_response(),
    }
}
