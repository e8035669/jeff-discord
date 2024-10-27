use super::common::Context;
use crate::entities_mod::global_pref_mod;
use crate::entities_mod::global_pref_mod::ActivType as DbActivType;
use crate::entities_mod::GlobalPrefMod;
use anyhow::Result;
use poise::serenity_prelude::Context as SerenityContext;
use poise::serenity_prelude::*;
use sea_orm::{ActiveModelTrait, Set, TryIntoModel};
use sea_orm::{DatabaseConnection, EntityTrait, IntoActiveModel};

#[derive(Debug, poise::ChoiceParameter)]
pub enum ActivType {
    Playing,
    Streaming,
    Listening,
    Watching,
    Custom,
    Competing,
}

impl From<ActivType> for DbActivType {
    fn from(activ_type: ActivType) -> Self {
        match activ_type {
            ActivType::Playing => DbActivType::Playing,
            ActivType::Streaming => DbActivType::Streaming,
            ActivType::Listening => DbActivType::Listening,
            ActivType::Watching => DbActivType::Watching,
            ActivType::Custom => DbActivType::Custom,
            ActivType::Competing => DbActivType::Competing,
        }
    }
}

/// 設定機器人的狀態
#[poise::command(slash_command, prefix_command, category = "admin", owners_only)]
pub async fn set_activity(
    ctx: Context<'_>,
    activ_type: Option<ActivType>,
    activ_msg: Option<String>,
    activ_url: Option<String>,
) -> Result<()> {
    ctx.defer().await?;
    let pref = get_pref_model(&ctx.data().pool).await?;
    let mut pref = pref.into_active_model();
    pref.activ_type = Set(activ_type.map(Into::into));
    pref.activ_msg = Set(activ_msg);
    pref.activ_url = Set(activ_url);
    let pref = pref.save(&ctx.data().pool).await?;
    let pref = pref.try_into_model()?;
    set_activity_from_db(&ctx.serenity_context(), &pref).await?;
    ctx.reply("OK").await?;
    Ok(())
}

pub async fn set_activity_from_db(
    ctx: &SerenityContext,
    pref: &global_pref_mod::Model,
) -> Result<()> {
    if let Some(activ_type) = &pref.activ_type {
        let msg = pref.activ_msg.clone().unwrap_or_default();
        let url = pref.activ_url.clone().unwrap_or_default();

        let activ_data = match activ_type {
            DbActivType::Playing => ActivityData::playing(msg),
            DbActivType::Streaming => ActivityData::streaming(msg, url)?,
            DbActivType::Listening => ActivityData::listening(msg),
            DbActivType::Watching => ActivityData::watching(msg),
            DbActivType::Custom => ActivityData::custom(msg),
            DbActivType::Competing => ActivityData::competing(msg),
        };

        ctx.set_activity(Some(activ_data));
    } else {
        ctx.set_activity(None);
    }
    Ok(())
}

pub async fn get_pref_model(db: &DatabaseConnection) -> Result<global_pref_mod::Model> {
    let model_opt = GlobalPrefMod::find_by_id(1).one(db).await?;
    if let Some(model) = model_opt {
        Ok(model)
    } else {
        let m = global_pref_mod::ActiveModel {
            id: Set(1),
            ..Default::default()
        };
        let model = m.insert(db).await?;
        Ok(model)
    }
}

/// 設定write指令的system prompt
#[poise::command(slash_command, prefix_command, category = "admin", owners_only)]
pub async fn set_write_system_prompt(
    ctx: Context<'_>,
    system_prompt: Option<String>,
) -> Result<()> {
    ctx.defer().await?;
    let pref = get_pref_model(&ctx.data().pool).await?;
    let mut pref = pref.into_active_model();
    pref.write_system_prompt = Set(system_prompt);
    pref.save(&ctx.data().pool).await?;
    ctx.reply("OK").await?;
    Ok(())
}

/// 顯示目前write指令的system prompt
#[poise::command(slash_command, prefix_command, category = "admin", owners_only)]
pub async fn get_write_system_prompt(ctx: Context<'_>) -> Result<()> {
    ctx.defer().await?;
    let pref = get_pref_model(&ctx.data().pool).await?;
    if let Some(prompt) = pref.write_system_prompt {
        ctx.reply(format!("```text\n{}\n```\n", prompt)).await?;
    } else {
        ctx.reply("未設定").await?;
    }
    Ok(())
}
