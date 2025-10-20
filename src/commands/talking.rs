use super::admin::get_pref_model;
use super::{common::Context, BotError, Data};
use anyhow::{anyhow, Error, Result};
use async_openai::error::OpenAIError;
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    ChatCompletionTool, ChatCompletionToolArgs, ChatCompletionToolType,
    CreateChatCompletionRequestArgs, FunctionObject, FunctionObjectArgs,
};
use chrono::Utc;
use poise::serenity_prelude::{
    self as serenity, ChannelId, ChannelType, Color, CreateEmbed, CreateEmbedFooter, CreateThread,
    EditChannel, FullEvent, GetMessages, GuildChannel, Message,
};
use poise::{CreateReply, FrameworkContext};
use rand::seq::IteratorRandom;
use sea_orm::{EnumIter, Iterable};
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

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

    let data = ctx.data();

    let pref = get_pref_model(&data.pool).await?;
    let system_prompt = pref
        .write_system_prompt
        .unwrap_or_else(|| SYSTEM_PROMPT.to_string());

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt.clone())
                .build()?
                .into(),
        ])
        .build()?;

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

static DEFAULT_THREAD_NAME: &str = "New chat room";

/// 開始跟機器人聊天
#[poise::command(
    prefix_command,
    slash_command,
    category = "talking",
    guild_only,
    owners_only
)]
pub async fn chat(ctx: Context<'_>) -> Result<()> {
    let channel = ctx
        .guild_channel()
        .await
        .ok_or(BotError::MessageNotFromGuild)?;
    let reply = ctx.reply("New chat created.").await?;
    let _new_thread = channel
        .create_thread_from_message(
            &ctx,
            reply.message().await?.id,
            CreateThread::new(DEFAULT_THREAD_NAME),
        )
        .await?;
    Ok(())
}

static TITLE_PROMPT_TEMPLATE: &str = r#"Create a concise, 3-5 word title with an emoji as a title for the chat history, in the given language. Suitable Emojis for the summary can be used to enhance understanding but avoid quotation marks or special formatting. RESPOND ONLY WITH THE TITLE TEXT.

Examples of titles:
📉 Stock Market Trends
🍪 Perfect Chocolate Chip Recipe
Evolution of Music Streaming
Remote Work Productivity Tips
Artificial Intelligence in Healthcare
🎮 Video Game Development Insights

<chat_history>
{messages}
</chat_history>"#;

pub async fn handle_chat_message(
    ctx: &serenity::Context,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
    thread_channel: &GuildChannel,
) -> Result<()> {
    let _typing = thread_channel.start_typing(&ctx.http);

    let mut history = thread_channel.messages(ctx, GetMessages::default()).await?;

    history.sort_by(|m1, m2| m1.timestamp.cmp(&m2.timestamp));

    for m in &history {
        let content = m.content.clone();
        let author = m.author.name.clone();
        println!("{} says: {}", author, content);
    }

    let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

    for m in &history {
        if m.content.len() == 0 {
            continue;
        }
        if m.author.id == framework.bot_id {
            let msg = ChatCompletionRequestAssistantMessageArgs::default()
                .content(m.content.clone())
                .build()?;
            messages.push(msg.into());
        } else {
            let msg = ChatCompletionRequestUserMessageArgs::default()
                .content(m.content.clone())
                .build()?;
            messages.push(msg.into());
        }
    }

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt")
        .messages(messages)
        .build()?;

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

    thread_channel.say(ctx, resp_msg).await?;

    if thread_channel.name() == DEFAULT_THREAD_NAME {
        let mut msg = String::new();
        for m in &history {
            if m.content.len() > 0 {
                msg.push_str(&m.content);
            }
        }

        let title_request = CreateChatCompletionRequestArgs::default()
            .model("gpt")
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(format!(r#"Create a concise, 3-5 word title with an emoji as a title for the chat history, in the given language. Suitable Emojis for the summary can be used to enhance understanding but avoid quotation marks or special formatting. RESPOND ONLY WITH THE TITLE TEXT.

Examples of titles:
📉 Stock Market Trends
🍪 Perfect Chocolate Chip Recipe
Evolution of Music Streaming
Remote Work Productivity Tips
Artificial Intelligence in Healthcare
🎮 Video Game Development Insights

<chat_history>
{}
</chat_history>"#, msg))
                .build()?
                .into()])
            .build()?;

        let title_resp = data.openai.chat().create(title_request).await?;
        let mut new_title = String::new();
        for c in &title_resp.choices {
            if let Some(c) = &c.message.content {
                new_title.push_str(c);
            }
        }

        let mut thread_channel = thread_channel.clone();
        thread_channel
            .edit(ctx, EditChannel::new().name(new_title))
            .await?;
        //
    }

    Ok(())
}

pub async fn handle_message(
    ctx: &serenity::Context,
    _event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
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

    if let Some(b) = &start_message.interaction {
        if b.name == "chat" {
            info!("Need handle chat message");
        } else {
            return Ok(());
        }
    } else {
        return Ok(());
    }

    // println!("Message: {:?}", message);
    // println!("guild_channel: {:?}", thread_channel);
    // println!("Start message: {:?}", start_message);

    if let Err(e) = handle_chat_message(ctx, framework, data, &thread_channel).await {
        warn!("Error handling chat message {}", e);
    }

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
        .choose(&mut rand::rng())
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

fn get_current_utc_datetime_tool() -> Result<ChatCompletionTool, OpenAIError> {
    ChatCompletionToolArgs::default()
        .r#type(ChatCompletionToolType::Function)
        .function(get_current_utc_datetime_func()?)
        .build()
}

fn get_current_utc_datetime_func() -> Result<FunctionObject, OpenAIError> {
    FunctionObjectArgs::default()
        .name("get_current_utc_datetime")
        .description("Get current UTC date and time.")
        .parameters(json!({
            "type": "object",
            "properties": {},
            "required": [],
        }))
        .build()
}

fn get_current_utc_datetime() -> serde_json::Value {
    let datetime = Utc::now();

    let ret = json!({
        "current_utc_date": datetime.date_naive(),
        "current_utc_time": datetime.time(),
    });

    ret
}
