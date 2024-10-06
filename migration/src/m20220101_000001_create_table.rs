use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(ColorRandomData::Table)
                    .if_not_exists()
                    .col(pk_auto(ColorRandomData::Id))
                    .col(big_unsigned(ColorRandomData::Guild))
                    .col(big_unsigned(ColorRandomData::Role))
                    .col(integer(ColorRandomData::Shift))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(ColorRandomData::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum ColorRandomData {
    Table,
    Id,
    Guild,
    Role,
    Shift,
}
