use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

#[command]
async fn botsend(_ctx: &Context, _msg: &Message) -> CommandResult {
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
