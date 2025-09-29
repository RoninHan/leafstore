use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(pk_uuid(Users::Id))
                    .col(ColumnDef::new(Users::Name).string())
                    .col(ColumnDef::new(Users::Sex).integer())
                    .col(ColumnDef::new(Users::Birthday).timestamp_with_time_zone())
                    .col(ColumnDef::new(Users::Phone).string())
                    .col(ColumnDef::new(Users::Email).string())
                    .col(ColumnDef::new(Users::AppId).string().not_null())
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建Blocks表
        manager
            .create_table(
                Table::create()
                    .table(Blocks::Table)
                    .if_not_exists()
                    .col(pk_uuid(Blocks::Id))
                    .col(ColumnDef::new(Blocks::Pid).string())
                    .col(ColumnDef::new(Blocks::Context).string())
                    .col(ColumnDef::new(Blocks::Imgs).json())
                    .col(ColumnDef::new(Blocks::Location).string())
                    .col(ColumnDef::new(Blocks::LatitudeAndLongitude).string())
                    .col(ColumnDef::new(Blocks::Draft).boolean())
                    .col(ColumnDef::new(Blocks::CreateTime).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(Blocks::UpdateTime).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        // 创建SearchHistory表
        manager
            .create_table(
                Table::create()
                    .table(SearchHistory::Table)
                    .if_not_exists()
                    .col(pk_uuid(SearchHistory::Id))
                    .col(ColumnDef::new(SearchHistory::Uid).uuid())
                    .col(ColumnDef::new(SearchHistory::History).json())
                    .col(ColumnDef::new(SearchHistory::CreateTime).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(SearchHistory::UpdateTime).timestamp_with_time_zone().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        // 删除Blocks表
        manager
            .drop_table(Table::drop().table(Blocks::Table).to_owned())
            .await?;

        // 删除SearchHistory表
        manager
            .drop_table(Table::drop().table(SearchHistory::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Name,
    Sex,
    Birthday,
    Phone,
    Email,
    AppId,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Blocks {
    Table,
    Id,
    Pid,
    Context,
    Imgs,
    Location,
    LatitudeAndLongitude,
    Draft,
    CreateTime,
    UpdateTime,
}

#[derive(DeriveIden)]
enum SearchHistory {
    Table,
    Id,
    Uid,
    History,
    CreateTime,
    UpdateTime,
}
