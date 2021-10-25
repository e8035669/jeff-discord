use tracing::warn;

use serenity::framework::standard::{
    macros::{command, group},
    Args, CommandResult,
};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[group]
#[commands(botsend, ping)]
struct Talking;

#[command]
#[owners_only]
async fn botsend(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = args.single::<u64>()?;
    let message = args.remains().ok_or("Empty message")?;
    let channel = ctx.http.get_channel(channel_id).await?;

    if let Err(why) = channel.id().say(&ctx.http, message).await {
        println!("Error sending message: {:?}", why);
    }
    Ok(())
}

#[command]
async fn ping(_ctx: &Context, msg: &Message) -> CommandResult {
    let response = MessageBuilder::new()
        .push("Hello ")
        .mention(&msg.author)
        .build();
    warn!("Hello");
    if let Err(why) = msg.channel_id.say(&_ctx.http, response).await {
        warn!("Error sending message: {:?}", why);
    }
    warn!("Hello");

    Ok(())
}
