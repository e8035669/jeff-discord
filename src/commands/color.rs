use super::common::Context;
use super::error::BotError;
use crate::entities::color_random_data;
use crate::entities::prelude::*;
use anyhow::Result;
use chrono::prelude::*;
use colorsys::{Hsl, HslRatio, Rgb};
use poise::serenity_prelude::{CacheHttp, Colour, EditRole, Guild, MessageBuilder, RoleId};
use sea_orm::ActiveModelTrait;
use sea_orm::IntoActiveModel;
use sea_orm::Set;
use sea_orm::{ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::debug;
use tracing::warn;

/// 註冊一個身份組，將在每天午夜換上新的顏色
#[poise::command(slash_command, prefix_command, category = "color")]
pub async fn colorreg(_ctx: Context<'_>, role_id: RoleId) -> Result<()> {
    let guild_id = _ctx.guild_id().ok_or(BotError::MessageNotFromGuild)?;

    if !_ctx
        .guild()
        .ok_or(BotError::GuildNotFound)?
        .roles
        .contains_key(&role_id)
    {
        return Err(BotError::RoleNotInGuild.into());
    }

    let color_data = get_color_data(&_ctx).await;
    debug!("reg role: {}, {}", guild_id.get(), role_id.get());
    let result = color_data.reg_role(guild_id.get(), role_id.get()).await?;
    if result {
        _ctx.say("OK").await?;
    } else {
        _ctx.say("record exists").await?;
    }

    update_all_colors(&_ctx, &color_data).await?;

    Ok(())
}

/// 解除註冊身份組，將不會再更換顏色
#[poise::command(slash_command, prefix_command, category = "color")]
pub async fn colorunreg(_ctx: Context<'_>, role_id: RoleId) -> Result<()> {
    let guild_id = _ctx.guild_id().ok_or(BotError::MessageNotFromGuild)?;

    let color_data = get_color_data(&_ctx).await;
    debug!("unreg_role: {}, {}", guild_id.get(), role_id.get());
    let result = color_data.unreg_role(guild_id.get(), role_id.get()).await?;
    if result {
        _ctx.say("OK").await?;
    } else {
        _ctx.say("record not exists").await?;
    }

    update_all_colors(&_ctx, &color_data).await?;

    Ok(())
}

/// 馬上換新的顏色，可以在顏色不好看的時候使用
#[poise::command(slash_command, prefix_command, category = "color")]
pub async fn nextcolor(_ctx: Context<'_>, role_id: RoleId) -> Result<()> {
    let guild_id = _ctx.guild_id().ok_or(BotError::MessageNotFromGuild)?;

    let color_data = get_color_data(&_ctx).await;
    debug!("nextcolor: {}, {}", guild_id.get(), role_id.get());
    let result = color_data.next_color(guild_id.get(), role_id.get()).await?;
    if result {
        _ctx.say("OK").await?;
    } else {
        _ctx.say("record not exists").await?;
    }

    update_all_colors(&_ctx, &color_data).await?;

    Ok(())
}

/// 列出已經註冊的身份組
#[poise::command(slash_command, prefix_command, category = "color")]
pub async fn listregs(_ctx: Context<'_>) -> Result<()> {
    let color_data = _ctx.data().color_data.clone();
    let data = color_data.get_reg_list().await?;

    let mut mb = MessageBuilder::new();
    mb.push("List of regs:\n");

    let cache = _ctx.cache();

    for (i, d) in data.iter().enumerate() {
        let guild = cache.guild(d.guild as u64).ok_or(BotError::GuildNotFound)?;
        let role = guild
            .roles
            .get(&RoleId::new(d.role as u64))
            .ok_or(BotError::RoleNotFound)?;

        mb.push(format!(
            "{}. [{}] [{}] offset:{}\n",
            i, guild.name, role.name, d.shift
        ));
    }

    _ctx.say(mb.build()).await?;
    Ok(())
}

async fn update_all_colors<T>(_ctx: &T, color_data: &ColorRandom) -> Result<()>
where
    T: CacheHttp,
{
    debug!("update all colors");
    let colors = color_data.get_all_colors().await?;

    let cache = _ctx.cache().ok_or(BotError::CacheFail)?;
    for c in colors.iter() {
        let guild_id = c.guild;
        let role_id = c.role;
        let color = c.color;

        let guild: Guild = cache
            .guild(guild_id)
            .ok_or(BotError::GuildNotFound)?
            .clone();
        let role = guild.roles.get(&RoleId::new(role_id));

        match role {
            Some(role) => {
                let mut role = role.clone();
                if role.colour != color {
                    let builder = EditRole::from_role(&role).colour(color.0 as u64);
                    if let Err(why) = role.edit(_ctx.http(), builder).await {
                        warn!("Cannot edit role {:?}", why);
                    }
                }
            }
            None => warn!("Cannot get role:"),
        }
    }

    Ok(())
}

async fn get_color_data(_ctx: &Context<'_>) -> Arc<ColorRandom> {
    _ctx.data().color_data.clone()
}

pub async fn update_loop<T>(_ctx: &T, color_data: &ColorRandom)
where
    T: CacheHttp,
{
    loop {
        if let Err(why) = update_all_colors(_ctx, color_data).await {
            warn!("update loop error: {:?}", why);
        }
        let wait_sec = color_data.get_waiting_time();
        debug!("Wait for {} seconds for next loop", wait_sec);
        sleep(Duration::new(wait_sec as u64, 0)).await;
    }
}

pub struct ColorRecord2 {
    guild: u64,
    role: u64,
    color: Colour,
}

pub struct ShiftRecord2 {
    guild: u64,
    role: u64,
    shift: i32,
}

impl From<color_random_data::Model> for ShiftRecord2 {
    fn from(m: color_random_data::Model) -> Self {
        Self {
            guild: m.guild as u64,
            role: m.role as u64,
            shift: m.shift,
        }
    }
}

pub trait ColorRandomTrait {
    fn get_reg_list(&self) -> impl std::future::Future<Output = Result<Vec<ShiftRecord2>>> + Send;

    fn get_all_colors(&self)
        -> impl std::future::Future<Output = Result<Vec<ColorRecord2>>> + Send;

    fn check_exists(
        &self,
        guild: u64,
        role: u64,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn reg_role(
        &self,
        guild: u64,
        role: u64,
    ) -> impl std::future::Future<Output = Result<bool>> + Send;

    fn unreg_role(&self, guild: u64, role: u64) -> impl std::future::Future<Output = Result<bool>>;

    fn next_color(&self, guild: u64, role: u64) -> impl std::future::Future<Output = Result<bool>>;

    fn get_waiting_time(&self) -> u32;
}

pub struct ColorRandom {
    db: sea_orm::DatabaseConnection,
    gmt8: FixedOffset,
}

impl ColorRandom {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self {
            db,
            gmt8: FixedOffset::east_opt(8 * 3600).unwrap(),
        }
    }

    fn convert(guild: u64, role: u64) -> (i64, i64) {
        (guild as i64, role as i64)
    }

    pub fn get_color(&self, role: i64, offset: i32) -> Colour {
        get_hashed_color(role, offset, self.today_ord())
    }

    pub fn today_ord(&self) -> i32 {
        self.today().num_days_from_ce()
    }

    pub fn today(&self) -> NaiveDate {
        Utc::now().with_timezone(&self.gmt8).date_naive()
    }
}

impl ColorRandomTrait for ColorRandom {
    async fn get_reg_list(&self) -> Result<Vec<ShiftRecord2>> {
        let data = ColorRandomData::find().all(&self.db).await?;
        let data2: Vec<ShiftRecord2> = data
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<ShiftRecord2>>();
        Ok(data2)
    }

    async fn get_all_colors(&self) -> Result<Vec<ColorRecord2>> {
        let reg_list = self.get_reg_list().await?;
        let ret = reg_list
            .into_iter()
            .map(|r| ColorRecord2 {
                guild: r.guild,
                role: r.role,
                color: self.get_color(r.role as i64, r.shift),
            })
            .collect::<Vec<_>>();

        Ok(ret)
    }

    async fn check_exists(&self, guild: u64, role: u64) -> Result<bool> {
        let (guild, role) = Self::convert(guild, role);
        let count = ColorRandomData::find()
            .filter(color_random_data::Column::Guild.eq(guild))
            .filter(color_random_data::Column::Role.eq(role))
            .count(&self.db)
            .await?;
        Ok(count > 0)
    }

    async fn reg_role(&self, guild: u64, role: u64) -> Result<bool> {
        if self.check_exists(guild, role).await? {
            Ok(false)
        } else {
            let (guild, role) = Self::convert(guild, role);
            let data = color_random_data::ActiveModel {
                guild: Set(guild),
                role: Set(role),
                shift: Set(0),
                ..Default::default()
            };
            ColorRandomData::insert(data).exec(&self.db).await?;
            Ok(true)
        }
    }

    async fn unreg_role(&self, guild: u64, role: u64) -> Result<bool> {
        let (guild, role) = Self::convert(guild, role);
        let roles = ColorRandomData::find()
            .filter(color_random_data::Column::Guild.eq(guild))
            .filter(color_random_data::Column::Role.eq(role))
            .all(&self.db)
            .await?;
        if roles.len() > 0 {
            for m in &roles {
                let ret = ColorRandomData::delete_by_id(m.id).exec(&self.db).await;
                if let Err(e) = ret {
                    warn!("unreg_role error {:?}", e);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn next_color(&self, guild: u64, role: u64) -> Result<bool> {
        let (guild, role) = Self::convert(guild, role);
        let rec = ColorRandomData::find()
            .filter(color_random_data::Column::Guild.eq(guild))
            .filter(color_random_data::Column::Role.eq(role))
            .one(&self.db)
            .await?;
        if let Some(m) = rec {
            let shift = m.shift;
            let mut am = m.into_active_model();
            am.shift = Set(shift + 1);
            am.save(&self.db).await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn get_waiting_time(&self) -> u32 {
        let secs = Utc::now()
            .with_timezone(&self.gmt8)
            .num_seconds_from_midnight();
        86400 - secs + 30
    }
}

#[derive(Hash)]
struct RandomTuple(i64, i32, i32);

fn get_random_color(mut id: u64) -> Colour {
    id %= 1000000;
    let hue = (id as f64 * 0.618033988749895) % 1.0;
    let sat = ((id as f64 * 0.377846739793041) % 0.8) + 0.2;
    let light = ((id as f64 * 0.7726261498488001) % 0.5) + 0.4;

    let hsl: Hsl = HslRatio::from((hue, sat, light)).into();
    let rgb: Rgb = hsl.into();
    let (r, g, b): (f64, f64, f64) = rgb.into();
    let new_color = Colour::from_rgb(r.round() as u8, g.round() as u8, b.round() as u8);
    new_color
}

fn get_hash(role: i64, offset: i32, day_ord: i32) -> u64 {
    let mut hasher = DefaultHasher::new();
    let t = RandomTuple(role, offset, day_ord);
    t.hash(&mut hasher);
    let ret = hasher.finish();
    ret
}

fn get_hashed_color(role: i64, offset: i32, day_ord: i32) -> Colour {
    get_random_color(get_hash(role, offset, day_ord))
}
