use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransferHistory::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TransferHistory::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TransferHistory::Pid).uuid().not_null())
                    .col(
                        ColumnDef::new(TransferHistory::TransferId)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferHistory::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(TransferHistory::FileName)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferHistory::FileSize)
                            .big_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(TransferHistory::FileType).string())
                    .col(ColumnDef::new(TransferHistory::RecipientName).string())
                    .col(ColumnDef::new(TransferHistory::RecipientIp).string())
                    .col(
                        ColumnDef::new(TransferHistory::CompletedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TransferHistory::CreatedAt)
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
                    .name("idx-history-transfer_id")
                    .table(TransferHistory::Table)
                    .col(TransferHistory::TransferId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-history-user_id")
                    .table(TransferHistory::Table)
                    .col(TransferHistory::UserId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransferHistory::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum TransferHistory {
    Table,
    Id,
    Pid,
    TransferId,
    UserId,
    FileName,
    FileSize,
    FileType,
    RecipientName,
    RecipientIp,
    CompletedAt,
    CreatedAt,
}

