use super::admin::get_pref_model;
use super::{common::Context, BotError, Data};
use anyhow::{anyhow, Error, Result};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use poise::serenity_prelude::{
    self as serenity, ChannelId, ChannelType, Color, CreateEmbed, CreateEmbedFooter, CreateThread,
    FullEvent, GetMessages, Message,
};
use poise::{CreateReply, FrameworkContext};
use rand::seq::IteratorRandom;
use sea_orm::{EnumIter, Iterable};
use std::time::Instant;
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

static SYSTEM_PROMPT: &str = r#"
You are Gemma AI, a friendly AI Assistant.
Respond to the input as a friendly AI assistant, generating human-like text, and follow the instructions in the input if applicable.
Keep the response concise and engaging, using Markdown when appropriate.
The user live in Taiwan, so be aware of the local context and preferences.
Use a conversational tone and provide helpful and informative responses, utilizing external knowledge when necessary.
"#;

/// 向機器人提問
#[poise::command(prefix_command, slash_command, category = "talking", owners_only)]
pub async fn write(ctx: Context<'_>, prompt: String) -> Result<()> {
    ctx.defer().await?;

    let t1 = Instant::now();

    let pref = get_pref_model(&ctx.data().pool).await?;
    let system_prompt = pref
        .write_system_prompt
        .unwrap_or_else(|| SYSTEM_PROMPT.to_string());

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt")
        .messages([
            ChatCompletionRequestAssistantMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt.clone())
                .build()?
                .into(),
        ])
        .build()?;

    let data = ctx.data();

    let response = data.openai.chat().create(request).await?;

    let mut resp_msg = String::new();

    for choice in &response.choices {
        let index = choice.index;
        let role = choice.message.role;
        let content = &choice.message.content;
        println!("{}: Role: {} Content: {:?}", index, role, content);

        if let Some(s) = content {
            resp_msg.push_str(s.as_str());
        }
    }

    let bot_resp = CreateReply::default().embed(
        CreateEmbed::default()
            .title(format!("> {}", prompt))
            // .description(format!("```text\n{}\n```\n", resp_msg)),
            .description(format!("{}\n", resp_msg))
            .color(Color::BLUE)
            .footer(CreateEmbedFooter::new(format!(
                "Process time: {:.2}s",
                t1.elapsed().as_secs_f32()
            ))),
    );
    ctx.send(bot_resp).await?;
    Ok(())
}

/// 開始跟機器人聊天
#[poise::command(
    prefix_command,
    slash_command,
    category = "talking",
    guild_only,
    owners_only
)]
pub async fn chat(ctx: Context<'_>, msg: String) -> Result<()> {
    let channel = ctx
        .guild_channel()
        .await
        .ok_or(BotError::MessageNotFromGuild)?;
    let reply = ctx.reply(format!("Message id is: {}", ctx.id())).await?;
    let new_thread = channel
        .create_thread_from_message(
            &ctx,
            reply.message().await?.id,
            CreateThread::new(msg.clone()),
        )
        .await?;
    new_thread.say(&ctx, "Hello~").await?;
    Ok(())
}

pub async fn handle_message(
    ctx: &serenity::Context,
    _event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    _data: &Data,
    message: &Message,
) -> Result<()> {
    if message.author.bot {
        return Ok(());
    }

    let channel = message.channel_id.to_channel(ctx).await?;
    let thread_channel = match channel.guild() {
        Some(c) => c,
        None => return Ok(()),
    };

    match thread_channel.kind {
        ChannelType::PrivateThread | ChannelType::PublicThread => {}
        _ => return Ok(()),
    }

    match thread_channel.owner_id {
        Some(i) => {
            if i != framework.bot_id {
                return Ok(());
            }
        }
        None => return Ok(()),
    }

    let guild_channel_id = thread_channel
        .parent_id
        .ok_or(BotError::ParentChannelNotFound)?;

    let start_message = guild_channel_id
        .message(ctx, thread_channel.id.get())
        .await?;

    println!("Message: {:?}", message);
    println!("guild_channel: {:?}", thread_channel);
    println!("Start message: {:?}", start_message);

    let mut history = thread_channel.messages(ctx, GetMessages::default()).await?;

    history.sort_by(|m1, m2| m1.timestamp.cmp(&m2.timestamp));

    for m in &history {
        let content = m.content.clone();
        let author = m.author.name.clone();
        println!("{} says: {}", author, content);
    }

    // if let Some(m) = history.first() {
    //     println!("First message: {:?}", m);
    // }

    Ok(())
}

#[derive(Debug, poise::ChoiceParameter, EnumIter, Clone, Copy, PartialEq, Eq)]
pub enum Rps {
    #[name = "布"]
    Paper = 0,
    #[name = "剪刀"]
    Scissors = 1,
    #[name = "石頭"]
    Stone = 2,
}

impl Rps {
    fn to_name(&self) -> String {
        match self {
            Rps::Paper => String::from("布"),
            Rps::Scissors => String::from("剪刀"),
            Rps::Stone => String::from("石頭"),
        }
    }
}

/// 跟機器人玩猜拳
#[poise::command(prefix_command, slash_command, category = "talking")]
pub async fn paper_scissors_stone(ctx: Context<'_>, shoot: Rps) -> Result<()> {
    ctx.defer().await?;

    let mine = Rps::iter()
        .choose(&mut rand::thread_rng())
        .ok_or(anyhow!("random error"))?;

    let i = (mine as i32 - shoot as i32 + 3) % 3;
    let mut message = String::new();
    message.push_str(format!("我出的是... {}!\n", mine.to_name()).as_str());
    match i {
        0 => {
            message.push_str("平手!");
        }
        1 => {
            message.push_str("你輸了!");
        }
        2 => {
            message.push_str("你贏了!");
        }
        _ => {}
    }
    ctx.reply(message).await?;
    Ok(())
}
