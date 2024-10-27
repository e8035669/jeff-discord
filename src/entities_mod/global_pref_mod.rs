use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "global_pref")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(column_type = "custom(\"enum_text\")", nullable)]
    pub activ_type: Option<ActivType>,
    pub activ_msg: Option<String>,
    pub activ_url: Option<String>,
    pub write_system_prompt: Option<String>,
    pub chat_system_prompt: Option<String>,
    pub summary_system_prompt: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "String(StringLen::None)", rename_all = "snake_case")]
pub enum ActivType {
    Playing,
    Streaming,
    Listening,
    Watching,
    Custom,
    Competing,
}
