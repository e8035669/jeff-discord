use sea_orm::{EnumIter, Iterable};
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
                    .table(GlobalPref::Table)
                    .if_not_exists()
                    .col(pk_auto(GlobalPref::Id))
                    .col(enumeration_null(
                        GlobalPref::ActivType,
                        Alias::new("activ_type"),
                        ActivType::iter(),
                    ))
                    .col(string_null(GlobalPref::ActivMsg))
                    .col(string_null(GlobalPref::ActivUrl))
                    .col(string_null(GlobalPref::WriteSystemPrompt))
                    .col(string_null(GlobalPref::ChatSystemPrompt))
                    .col(string_null(GlobalPref::SummarySystemPrompt))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(GlobalPref::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum GlobalPref {
    Table,
    Id,
    ActivType,
    ActivMsg,
    ActivUrl,
    WriteSystemPrompt,
    ChatSystemPrompt,
    SummarySystemPrompt,
}

#[derive(Iden, EnumIter)]
pub enum ActivType {
    Playing,
    Streaming,
    Listening,
    Watching,
    Custom,
    Competing,
}
