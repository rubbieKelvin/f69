use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Projects::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Projects::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Projects::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Projects::OwnerUserId).uuid().not_null())
                    .col(
                        ColumnDef::new(Projects::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(FeatureFlags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(FeatureFlags::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(FeatureFlags::ProjectId).uuid().not_null())
                    .col(ColumnDef::new(FeatureFlags::Key).string_len(255).not_null())
                    .col(ColumnDef::new(FeatureFlags::Enabled).boolean().not_null())
                    .col(
                        ColumnDef::new(FeatureFlags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_flags_project")
                            .from(FeatureFlags::Table, FeatureFlags::ProjectId)
                            .to(Projects::Table, Projects::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_flags_project_key")
                    .table(FeatureFlags::Table)
                    .col(FeatureFlags::ProjectId)
                    .col(FeatureFlags::Key)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(FeatureFlags::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Projects::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Projects {
    Table,
    Id,
    Name,
    OwnerUserId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum FeatureFlags {
    Table,
    Id,
    ProjectId,
    Key,
    Enabled,
    CreatedAt,
}
