//! SeaORM migrations for the flags database (`projects`, `feature_flags`).

pub use sea_orm_migration::prelude::*;

mod m20240328_000001_create_projects_and_flags;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240328_000001_create_projects_and_flags::Migration)]
    }
}
