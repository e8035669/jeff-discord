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
async fn botsend(ctx: &Context, _msg: &Message, mut args: Args) -> CommandResult {
    let channel_id = args.single::<u64>()?;
    let message = args.remains().unwrap_or_default();

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
    if let Err(why) = msg.channel_id.say(&_ctx.http, response).await {
        println!("Error sending message: {:?}", why);
    }

    Ok(())
}
