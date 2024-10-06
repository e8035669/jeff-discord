use super::common::Context;
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, ChannelId};
use tracing::warn;

/// 讓機器人去某個頻道說話
#[poise::command(prefix_command, slash_command, category = "talking", owners_only)]
pub async fn botsend(
    ctx: Context<'_>,
    #[description = "Channel id"] channel_id: u64,
    #[description = "Message to send"] message: String,
) -> Result<()> {
    let channel = ctx
        .serenity_context()
        .http
        .get_channel(ChannelId::new(channel_id))
        .await?;

    if let Err(why) = channel.id().say(&ctx, message).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}

/// 跟機器人問早
#[poise::command(prefix_command, slash_command, category = "talking")]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    let response = serenity::MessageBuilder::new()
        .push("Hello ")
        .mention(ctx.author())
        .build();
    if let Err(why) = ctx.say(response).await {
        warn!("Error sending message: {:?}", why);
    }

    Ok(())
}

/*
/// 操控機器人說話的指令
#[group]
#[commands(botsend, ping)]
struct Talking;

/// 讓機器人去某個頻道說話
#[command]
#[owners_only]
#[usage = "<channel_id> <messages>..."]
async fn botsend(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = args.single::<u64>()?;
    let message = args.remains().ok_or("Empty message")?;
    let channel = ctx.http.get_channel(channel_id).await?;

    if let Err(why) = channel.id().say(&ctx.http, message).await {
        println!("Error sending message: {:?}", why);
    }
    Ok(())
}

/// 跟機器人問早
#[command]
async fn ping(_ctx: &Context, msg: &Message) -> CommandResult {
    let response = MessageBuilder::new()
        .push("Hello ")
        .mention(&msg.author)
        .build();
    if let Err(why) = msg.channel_id.say(&_ctx.http, response).await {
        warn!("Error sending message: {:?}", why);
    }

    Ok(())
}
*/
