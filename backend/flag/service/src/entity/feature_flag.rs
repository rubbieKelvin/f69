//! Boolean flag belonging to a single project (unique `(project_id, key)` in the DB).

use sea_orm::entity::prelude::*;

use super::project;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "feature_flags")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: Uuid,
    pub key: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "project::Entity",
        from = "Column::ProjectId",
        to = "project::Column::Id"
    )]
    Project,
}

impl Related<project::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Project.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
