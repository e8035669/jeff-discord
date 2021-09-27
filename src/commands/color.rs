use std::error::Error;
use std::sync::Arc;
use tracing::debug;
use tracing::warn;

use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use sqlx::{Pool, Postgres};

#[group]
#[commands(colorreg, colorunref, nextcolor, listregs)]
struct Color;

#[command]
async fn colorreg(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn colorunref(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
async fn nextcolor(_ctx: &Context, _msg: &Message) -> CommandResult {
    Ok(())
}

async fn _listregs(_ctx: &Context, _msg: &Message) -> CommandResult {
    let data = _ctx.data.read().await;

    if let Some(color_data) = data.get::<ColorRandomDataContainer>() {
        let data = color_data.lock().await.get_all_colors().await?;

        let mut mb = MessageBuilder::new();
        mb.push("List of regs:\n");
        for (i, d) in data.iter().enumerate() {
            let guild = _ctx.cache.guild(d.guild as u64).await.ok_or("no guild name")?;
            let role = _ctx.cache.role(guild.id, d.role as u64).await.ok_or("no role")?;

            mb.push(format!(
                "{}. [{}] [{}] offset:{}\n",
                i, guild.name, role.name, d.shift
            ));
        }
        _msg.channel_id.say(&_ctx.http, mb.build()).await?;
    }
    Ok(())
}

#[command]
async fn listregs(_ctx: &Context, _msg: &Message) -> CommandResult {
    debug!("listregs");
    if let Err(why) = _listregs(_ctx, _msg).await {
        warn!("Error listreg: {:?}", why);
    }
    Ok(())
}

pub struct ColorRandomDataContainer;

impl TypeMapKey for ColorRandomDataContainer {
    type Value = Arc<Mutex<ColorRandomData>>;
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
    //
}

#[allow(dead_code)]
pub struct ColorRandomData {
    pool: Pool<Postgres>,
}

#[allow(dead_code)]
impl ColorRandomData {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
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

    pub async fn check_exists(&self, guild: i64, role: i64) -> Result<bool, Box<dyn Error>> {
        let (count,): (i32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM color_random_data
             WHERE guild=$1 and role=$2",
        )
        .bind(guild)
        .bind(role)
        .fetch_one(&self.pool)
        .await?;

        Ok(count > 0)
    }

    pub async fn reg_role(&self, guild: i64, role: i64) -> Result<bool, Box<dyn Error>> {
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

    pub async fn unreg_role(&self, guild: i64, role: i64) -> Result<bool, Box<dyn Error>> {
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

    pub async fn next_color(&self, guild: i64, role: i64) -> Result<bool, Box<dyn Error>> {
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

    pub async fn get_all_colors(&self) -> Result<Vec<ShiftRecord>, sqlx::Error> {
        let ret = sqlx::query_as::<_, ShiftRecord>(
            "SELECT guild, role, shift
            FROM color_random_data",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(ret)
    }
}
