use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
};
use serenity::utils::parse_emoji;
use std::collections::HashMap;
use tracing::debug;

use serenity::model::prelude::*;
use serenity::prelude::*;

/// 統計某些東西
#[group]
#[commands(emojistat)]
struct Statistic;

/// 統計近30天的emoji使用
#[command]
#[owners_only]
async fn emojistat(_ctx: &Context, _msg: &Message) -> CommandResult {
    let _guild = _msg.guild(&_ctx.cache).ok_or("Guild not found in cache")?;

    let guild_emoji = _guild.emojis;

    let mut counts = HashMap::new();
    for (k, _) in guild_emoji.iter() {
        counts.insert(k.to_owned(), 0u32);
    }

    let current_time = Utc::now();

    let ch = _msg.channel(&_ctx).await?;
    if let Channel::Guild(ch) = ch {
        let old_time = current_time - Duration::seconds(60);

        let msgs = query_messages(&_ctx, &ch, old_time.into()).await?;

        let msgs2: Vec<_> = msgs.into_values().map(|m| m.content).collect();

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(<a?)?:\w+:(\d{18}>)?").unwrap();
        }

        let emojis: Vec<EmojiId> = msgs2
            .iter()
            .flat_map(|m| RE.find_iter(m).filter_map(|e| parse_emoji(e.as_str())))
            .filter(|e| guild_emoji.contains_key(&e.id))
            .map(|e| e.id)
            .collect();

        emojis.iter().for_each(|e| *counts.get_mut(e).unwrap() += 1);

        for (k, v) in counts.iter() {
            let e = guild_emoji.get(k).unwrap();
            debug!("{}: {}", e.name, v);
        }
    }

    Ok(())
}

async fn query_messages(
    _ctx: &Context,
    _ch: &GuildChannel,
    _after: Timestamp,
) -> CommandResult<HashMap<MessageId, Message>> {
    let mut ret = HashMap::new();
    let mut last_msg_id: Option<MessageId> = None;
    let mut is_enough = false;

    while !is_enough {
        let msgs = _ch
            .messages(_ctx, |m| {
                if let Some(i) = last_msg_id {
                    m.before(i);
                }
                m.limit(100)
            })
            .await?;

        if msgs.len() != 100 {
            is_enough = true
        }

        for m in msgs {
            last_msg_id = match last_msg_id {
                Some(i) => Some(i.min(m.id)),
                None => Some(m.id),
            };

            if m.timestamp > _after {
                ret.insert(m.id, m);
            } else {
                is_enough = true
            }
        }
    }

    Ok(ret)
}
