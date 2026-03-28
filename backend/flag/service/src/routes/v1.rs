use axum::extract::{Path, State};
use axum::Json;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::auth_user::AuthUser;
use crate::entity::{feature_flag, project};
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ProjectDto {
    pub id: Uuid,
    pub name: String,
    pub owner_user_id: Uuid,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
}

pub async fn list_projects(
    AuthUser { user_id: user }: AuthUser,
    State(state): State<AppState>,
) -> Result<Json<Vec<ProjectDto>>, ApiError> {
    let rows = project::Entity::find()
        .filter(project::Column::OwnerUserId.eq(user))
        .order_by_asc(project::Column::Name)
        .all(&state.db)
        .await?;
    let out = rows
        .into_iter()
        .map(|p| ProjectDto {
            id: p.id,
            name: p.name,
            owner_user_id: p.owner_user_id,
        })
        .collect();
    Ok(Json(out))
}

pub async fn create_project(
    AuthUser { user_id: user }: AuthUser,
    State(state): State<AppState>,
    Json(body): Json<CreateProjectRequest>,
) -> Result<Json<ProjectDto>, ApiError> {
    body.validate()
        .map_err(|_| ApiError::BadRequest("validation failed"))?;

    let id = Uuid::new_v4();
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let model = project::ActiveModel {
        id: Set(id),
        name: Set(body.name),
        owner_user_id: Set(user),
        created_at: Set(now),
    };
    let p = model.insert(&state.db).await?;
    Ok(Json(ProjectDto {
        id: p.id,
        name: p.name,
        owner_user_id: p.owner_user_id,
    }))
}

#[derive(Debug, Serialize)]
pub struct FlagDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub key: String,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateFlagRequest {
    #[validate(length(min = 1, max = 255))]
    pub key: String,
    #[serde(default)]
    pub enabled: bool,
}

pub async fn list_flags(
    AuthUser { user_id: user }: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
) -> Result<Json<Vec<FlagDto>>, ApiError> {
    let proj = project::Entity::find_by_id(project_id)
        .one(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;
    if proj.owner_user_id != user {
        return Err(ApiError::Forbidden);
    }

    let rows = feature_flag::Entity::find()
        .filter(feature_flag::Column::ProjectId.eq(project_id))
        .order_by_asc(feature_flag::Column::Key)
        .all(&state.db)
        .await?;
    let out = rows
        .into_iter()
        .map(|f| FlagDto {
            id: f.id,
            project_id: f.project_id,
            key: f.key,
            enabled: f.enabled,
        })
        .collect();
    Ok(Json(out))
}

pub async fn create_flag(
    AuthUser { user_id: user }: AuthUser,
    State(state): State<AppState>,
    Path(project_id): Path<Uuid>,
    Json(body): Json<CreateFlagRequest>,
) -> Result<Json<FlagDto>, ApiError> {
    body.validate()
        .map_err(|_| ApiError::BadRequest("validation failed"))?;

    let proj = project::Entity::find_by_id(project_id)
        .one(&state.db)
        .await?
        .ok_or(ApiError::NotFound)?;
    if proj.owner_user_id != user {
        return Err(ApiError::Forbidden);
    }

    let id = Uuid::new_v4();
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let model = feature_flag::ActiveModel {
        id: Set(id),
        project_id: Set(project_id),
        key: Set(body.key),
        enabled: Set(body.enabled),
        created_at: Set(now),
    };
    let f = model.insert(&state.db).await?;

    Ok(Json(FlagDto {
        id: f.id,
        project_id: f.project_id,
        key: f.key,
        enabled: f.enabled,
    }))
}
