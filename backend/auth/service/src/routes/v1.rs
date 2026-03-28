use axum::{extract::State, Json};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::entity::user;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub id: Uuid,
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}

pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, ApiError> {
    body.validate()
        .map_err(|_| ApiError::BadRequest("validation failed"))?;

    let exists = user::Entity::find()
        .filter(user::Column::Email.eq(body.email.to_lowercase()))
        .one(&state.db)
        .await?;
    if exists.is_some() {
        return Err(ApiError::Conflict("email already registered"));
    }

    let hash = shared_security::password::hash_password(&body.password)
        .map_err(|_| ApiError::Internal)?;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let model = user::ActiveModel {
        id: Set(id),
        email: Set(body.email.to_lowercase()),
        password_hash: Set(hash),
        created_at: Set(now),
    };
    model.insert(&state.db).await?;

    Ok(Json(RegisterResponse {
        id,
        email: body.email.to_lowercase(),
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, ApiError> {
    let user = user::Entity::find()
        .filter(user::Column::Email.eq(body.email.to_lowercase()))
        .one(&state.db)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    if !shared_security::password::verify_password(&user.password_hash, &body.password) {
        return Err(ApiError::Unauthorized);
    }

    let token = state
        .jwt
        .sign_user(&user.id)
        .map_err(|_| ApiError::Internal)?;

    Ok(Json(LoginResponse {
        access_token: token,
        token_type: "Bearer",
        expires_in: state.jwt_ttl_secs,
    }))
}
