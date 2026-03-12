use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transfers::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Transfers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Transfers::Pid).uuid().not_null())
                    .col(ColumnDef::new(Transfers::SenderId).integer().not_null())
                    .col(ColumnDef::new(Transfers::TransferId).string().not_null())
                    .col(ColumnDef::new(Transfers::FileName).string().not_null())
                    .col(ColumnDef::new(Transfers::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Transfers::FileType).string())
                    .col(ColumnDef::new(Transfers::Status).string().not_null())
                    .col(
                        ColumnDef::new(Transfers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Transfers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx-transfers-transfer_id")
                    .table(Transfers::Table)
                    .col(Transfers::TransferId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-transfers-sender_id")
                    .table(Transfers::Table)
                    .col(Transfers::SenderId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Transfers::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Transfers {
    Table,
    Id,
    Pid,
    SenderId,
    TransferId,
    FileName,
    FileSize,
    FileType,
    Status,
    CreatedAt,
    UpdatedAt,
}
