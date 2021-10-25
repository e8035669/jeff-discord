use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tracing::debug;
use tracing::warn;

use tokio::time::{sleep, Duration};

use chrono::prelude::*;

use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandError, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::Colour;
use serenity::utils::MessageBuilder;

use sqlx::{Pool, Postgres};

use colorsys::{Hsl, HslRatio, Rgb};

#[group]
#[commands(colorreg, colorunreg, nextcolor, listregs)]
struct Color;

async fn get_color_data(_ctx: &Context) -> Arc<ColorRandomData> {
    let data = _ctx.data.read().await;
    data.get::<ColorRandomDataContainer>()
        .expect("Cannot get color data")
        .clone()
}

#[command]
async fn colorreg(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = _msg
        .guild_id
        .ok_or(CommandError::from("message not from guild"))?;
    let role_id = _args.single::<RoleId>()?;

    if !_msg
        .guild(&_ctx.cache)
        .await
        .ok_or_else(|| CommandError::from("Guild not found"))?
        .roles
        .contains_key(&role_id)
    {
        return Err("Role not belong to this guild".into());
    }

    let color_data = get_color_data(_ctx).await;
    debug!("reg role: {}, {}", guild_id.0, role_id.0);
    let result = color_data
        .reg_role(guild_id.0 as i64, role_id.0 as i64)
        .await?;
    if result {
        _msg.reply(&_ctx.http, "OK").await?;
    } else {
        _msg.reply(&_ctx.http, "record exists").await?;
    }

    update_all_colors(_ctx).await?;

    Ok(())
}

#[command]
async fn colorunreg(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = _msg
        .guild_id
        .ok_or(CommandError::from("message not from guild"))?;
    let role_id = _args.single::<RoleId>()?;

    let color_data = get_color_data(_ctx).await;
    debug!("unreg_role: {}, {}", guild_id.0, role_id.0);
    let result = color_data
        .unreg_role(guild_id.0 as i64, role_id.0 as i64)
        .await?;
    if result {
        _msg.reply(&_ctx.http, "OK").await?;
    } else {
        _msg.reply(&_ctx.http, "record not exists").await?;
    }

    update_all_colors(_ctx).await?;

    Ok(())
}

#[command]
async fn nextcolor(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = _msg
        .guild_id
        .ok_or(CommandError::from("message not from guild"))?;
    let role_id = _args.single::<RoleId>()?;

    let color_data = get_color_data(_ctx).await;
    debug!("nextcolor: {}, {}", guild_id.0, role_id.0);
    let result = color_data
        .next_color(guild_id.0 as i64, role_id.0 as i64)
        .await?;
    if result {
        _msg.reply(&_ctx.http, "OK").await?;
    } else {
        _msg.reply(&_ctx.http, "record not exists").await?;
    }

    update_all_colors(_ctx).await?;

    Ok(())
}

#[command]
async fn listregs(_ctx: &Context, _msg: &Message) -> CommandResult {
    let data = _ctx.data.read().await;

    if let Some(color_data) = data.get::<ColorRandomDataContainer>() {
        let data = color_data.get_reg_list().await?;

        let mut mb = MessageBuilder::new();
        mb.push("List of regs:\n");
        for (i, d) in data.iter().enumerate() {
            let guild = _ctx
                .cache
                .guild(d.guild as u64)
                .await
                .ok_or("no guild name")?;
            let role = _ctx
                .cache
                .role(guild.id, d.role as u64)
                .await
                .ok_or("no role")?;

            mb.push(format!(
                "{}. [{}] [{}] offset:{}\n",
                i, guild.name, role.name, d.shift
            ));
        }
        _msg.channel_id.say(&_ctx.http, mb.build()).await?;
    }
    Ok(())
}

async fn update_all_colors(_ctx: &Context) -> CommandResult {
    debug!("update all colors");
    let color_data = get_color_data(_ctx).await;
    let colors = color_data.get_all_colors().await?;

    for c in colors.iter() {
        let guild_id = c.guild;
        let role_id = c.role;
        let color = c.color;

        let role = _ctx.cache.role(guild_id as u64, role_id as u64).await;

        match role {
            Some(role) => {
                if role.colour != color {
                    if let Err(why) = role.edit(&_ctx, |r| r.colour(color.0 as u64)).await {
                        warn!("Cannot edit role {:?}", why);
                    }
                }
            }
            None => warn!("Cannot get role:"),
        }
    }

    Ok(())
}

pub struct ColorRandomDataContainer;

impl TypeMapKey for ColorRandomDataContainer {
    type Value = Arc<ColorRandomData>;
}

#[allow(dead_code)]
#[derive(sqlx::FromRow)]
pub struct ShiftRecord {
    guild: i64,
    role: i64,
    shift: i32,
}

#[allow(dead_code)]
pub struct ColorRecord {
    guild: i64,
    role: i64,
    color: Colour,
}

#[allow(dead_code)]
pub struct ColorRandomData {
    pool: Pool<Postgres>,
    gmt8: FixedOffset,
}

#[allow(dead_code)]
impl ColorRandomData {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            gmt8: FixedOffset::east(8 * 3600),
        }
    }

    pub async fn init(&self) {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS color_random_data(
                guild bigint, role bigint, shift int) ",
        )
        .execute(&self.pool)
        .await
        .expect("Create table failed");
    }

    pub async fn check_exists(&self, guild: i64, role: i64) -> Result<bool, sqlx::Error> {
        let (count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM color_random_data
             WHERE guild=$1 and role=$2",
        )
        .bind(guild)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn reg_role(&self, guild: i64, role: i64) -> Result<bool, sqlx::Error> {
        if !self.check_exists(guild, role).await? {
            sqlx::query(
                "INSERT INTO color_random_data
                 VALUES($1, $2, 0)",
            )
            .bind(guild)
            .bind(role)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn unreg_role(&self, guild: i64, role: i64) -> Result<bool, sqlx::Error> {
        if self.check_exists(guild, role).await? {
            sqlx::query(
                "DELETE FROM color_random_data
                 WHERE guild=$1 and role=$2",
            )
            .bind(guild)
            .bind(role)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn next_color(&self, guild: i64, role: i64) -> Result<bool, sqlx::Error> {
        if self.check_exists(guild, role).await? {
            sqlx::query(
                "UPDATE color_random_data
                 SET shift = shift + 1
                 WHERE guild=$1 and role=$2",
            )
            .bind(guild)
            .bind(role)
            .execute(&self.pool)
            .await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn get_reg_list(&self) -> Result<Vec<ShiftRecord>, sqlx::Error> {
        let ret = sqlx::query_as::<_, ShiftRecord>(
            "SELECT guild, role, shift
            FROM color_random_data",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ret)
    }

    pub async fn get_all_colors(&self) -> Result<Vec<ColorRecord>, sqlx::Error> {
        let reg_list = self.get_reg_list().await?;
        let mut ret = Vec::new();
        for record in reg_list.iter() {
            ret.push(ColorRecord {
                guild: record.guild,
                role: record.role,
                color: self.get_color(record.role, record.shift),
            });
        }

        Ok(ret)
    }

    pub fn get_color(&self, role: i64, offset: i32) -> Colour {
        get_hashed_color(role, offset, self.today_ord())
    }

    pub fn today(&self) -> Date<FixedOffset> {
        Utc::now().with_timezone(&self.gmt8).date()
    }

    pub fn today_ord(&self) -> i32 {
        self.today().num_days_from_ce()
    }

    pub fn get_waiting_time(&self) -> u32 {
        let secs = Utc::now()
            .with_timezone(&self.gmt8)
            .num_seconds_from_midnight();
        86400 - secs + 30
    }

    pub async fn update_loop(&self, _ctx: &Context) {
        loop {
            if let Err(why) = update_all_colors(_ctx).await {
                warn!("update loop error: {:?}", why);
            }
            let wait_sec = self.get_waiting_time();
            debug!("Wait for {} seconds for next loop", wait_sec);
            sleep(Duration::new(wait_sec as u64, 0)).await;
        }
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
