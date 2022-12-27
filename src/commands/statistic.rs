use crate::{Context, Error};
use chrono::{Duration, Utc};
use lazy_static::lazy_static;
use poise::serenity_prelude::{
    parse_emoji, Channel, CreateActionRow, CreateEmbed, Emoji, EmojiId, GuildChannel,
    InteractionResponseType, Mentionable, Message, MessageId, ReactionType, Timestamp,
};
use regex::Regex;
use std::collections::HashMap;
use std::time::Duration as StdDuration;
use tracing::debug;

/// 統計近3天的emoji使用
#[poise::command(slash_command, prefix_command, category = "statistic", owners_only)]
pub async fn emojistat(
    _ctx: Context<'_>,
    #[description = "More days"] ndays: Option<i64>,
) -> Result<(), Error> {
    let _is_typing = _ctx.defer_or_broadcast().await?;

    let _guild = _ctx.guild().ok_or("Guild not found in cache")?;

    let guild_emoji = _guild.emojis;

    let mut counts = HashMap::new();
    for (k, _) in guild_emoji.iter() {
        counts.insert(k.to_owned(), 0u32);
    }

    let current_time = Utc::now();
    let old_time = current_time - Duration::days(ndays.unwrap_or(3));

    let channels = &_guild.channels;

    for ch in channels.values() {
        if let Channel::Guild(ch) = ch {
            let msgs = query_messages(&_ctx, &ch, old_time.into()).await?;

            let msgs = msgs.into_values().collect::<Vec<_>>();

            let msgs2: Vec<_> = msgs.iter().map(|m| m.content.to_owned()).collect();

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

            let react_emojis: Vec<(EmojiId, u32)> = msgs
                .iter()
                .flat_map(|m| m.reactions.clone())
                .filter_map(|r| {
                    if let ReactionType::Custom {
                        animated: _,
                        id,
                        name: _,
                    } = r.reaction_type
                    {
                        Some((id, r.count as u32))
                    } else {
                        None
                    }
                })
                .filter(|r| guild_emoji.contains_key(&r.0))
                .collect();

            react_emojis
                .iter()
                .for_each(|r| *counts.get_mut(&r.0).unwrap() += r.1);

            debug!("Collect {} messages", msgs.len());
        }
    }

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let mut messages: Vec<String> = Vec::new();
    for (e, c) in counts.iter() {
        let e2 = guild_emoji.get(e).unwrap();
        messages.push(format!("{}: {}", e2, c));
    }

    let sorted_count_chunks = counts.chunks(15).collect::<Vec<_>>();

    let mut current_page: isize = 0;

    let mut m = _ctx
        .channel_id()
        .send_message(&_ctx, |m| {
            m.set_embed(get_embed_message(
                &guild_emoji,
                &sorted_count_chunks,
                current_page,
            ))
            .components(|c| c.add_action_row(create_buttom_row()))
            .content(format!("{}", _ctx.author().mention()))
        })
        .await?;

    while let Some(interaction) = m
        .await_component_interaction(_ctx)
        .timeout(StdDuration::from_secs(60 * 10))
        .await
    {
        let custom_id = interaction.data.custom_id.as_str();
        match custom_id {
            "prev" | "next" => {
                let shift: isize = match custom_id {
                    "prev" => -1,
                    "next" => 1,
                    _ => 0,
                };

                current_page =
                    (current_page + shift).clamp(0, (sorted_count_chunks.len() - 1) as isize);
                interaction
                    .create_interaction_response(&_ctx, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| {
                                m.set_embed(get_embed_message(
                                    &guild_emoji,
                                    &sorted_count_chunks,
                                    current_page,
                                ))
                                .components(|c| c.add_action_row(create_buttom_row()))
                            })
                    })
                    .await?;
            }
            _ => break,
        }
    }

    m.edit(&_ctx, |m| m.components(|c| c)).await?;
    _ctx.say("Interaction Expired").await?;

    Ok(())
}

/*
/// 統計某些東西
#[group]
#[commands(emojistat)]
struct Statistic;

/// 統計近180天的emoji使用
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
    let old_time = current_time - Duration::days(365);

    let channels = &_guild.channels;

    for ch in channels.values() {
        if let Channel::Guild(ch) = ch {
            let msgs = query_messages(&_ctx, &ch, old_time.into()).await?;

            let msgs = msgs.into_values().collect::<Vec<_>>();

            let msgs2: Vec<_> = msgs.iter().map(|m| m.content.to_owned()).collect();

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

            let react_emojis: Vec<(EmojiId, u32)> = msgs
                .iter()
                .flat_map(|m| m.reactions.clone())
                .filter_map(|r| {
                    if let ReactionType::Custom {
                        animated: _,
                        id,
                        name: _,
                    } = r.reaction_type
                    {
                        Some((id, r.count as u32))
                    } else {
                        None
                    }
                })
                .filter(|r| guild_emoji.contains_key(&r.0))
                .collect();

            react_emojis
                .iter()
                .for_each(|r| *counts.get_mut(&r.0).unwrap() += r.1);

            debug!("Collect {} messages", msgs.len());
        }
    }

    let mut counts = counts.into_iter().collect::<Vec<_>>();
    counts.sort_by(|a, b| b.1.cmp(&a.1));

    let mut messages: Vec<String> = Vec::new();
    for (e, c) in counts.iter() {
        let e2 = guild_emoji.get(e).unwrap();
        messages.push(format!("{}: {}", e2, c));
    }

    let sorted_count_chunks = counts.chunks(15).collect::<Vec<_>>();

    let mut current_page: isize = 0;

    let mut m = _msg
        .channel_id
        .send_message(&_ctx, |m| {
            m.set_embed(get_embed_message(
                &guild_emoji,
                &sorted_count_chunks,
                current_page,
            ))
            .components(|c| c.add_action_row(create_buttom_row()))
            .content(format!("{}", _msg.author.mention()))
        })
        .await?;

    let mut interaction_stream = m
        .await_component_interactions(&_ctx)
        .timeout(StdDuration::from_secs(60 * 10))
        .build();

    while let Some(interaction) = interaction_stream.next().await {
        let custom_id = interaction.data.custom_id.as_str();
        match custom_id {
            "prev" | "next" => {
                let shift: isize = match custom_id {
                    "prev" => -1,
                    "next" => 1,
                    _ => 0,
                };

                current_page =
                    (current_page + shift).clamp(0, (sorted_count_chunks.len() - 1) as isize);
                interaction
                    .create_interaction_response(&_ctx, |r| {
                        r.kind(InteractionResponseType::UpdateMessage)
                            .interaction_response_data(|m| {
                                m.set_embed(get_embed_message(
                                    &guild_emoji,
                                    &sorted_count_chunks,
                                    current_page,
                                ))
                                .components(|c| c.add_action_row(create_buttom_row()))
                            })
                    })
                    .await?;
            }
            _ => break,
        }
    }

    m.edit(&_ctx, |m| m.components(|c| c)).await?;

    Ok(())
}
*/

fn get_embed_message(
    emojis: &HashMap<EmojiId, Emoji>,
    all_emoji_counts: &Vec<&[(EmojiId, u32)]>,
    page: isize,
) -> CreateEmbed {
    let mut embed = CreateEmbed::default();
    embed.title("Emoji Statistic Result");
    embed.description(get_message(emojis, all_emoji_counts, page));
    embed.footer(|f| f.text(format!("Page {}/{}", page + 1, all_emoji_counts.len())));

    embed
}

fn get_message(
    emojis: &HashMap<EmojiId, Emoji>,
    all_emoji_counts: &Vec<&[(EmojiId, u32)]>,
    page: isize,
) -> String {
    let emoji_counts = all_emoji_counts[page as usize];
    let ss = emoji_counts
        .iter()
        .map(|item| {
            let e = emojis.get(&item.0).unwrap();
            format!("{}: {}", e, item.1)
        })
        .collect::<Vec<String>>();
    ss.join("\n")
}

fn create_buttom_row() -> CreateActionRow {
    let mut r = CreateActionRow::default();
    r.create_button(|b| b.label("Prev").custom_id("prev"))
        .create_button(|b| b.label("Next").custom_id("next"));
    r
}

async fn query_messages(
    _ctx: &Context<'_>,
    _ch: &GuildChannel,
    _after: Timestamp,
) -> Result<HashMap<MessageId, Message>, Error> {
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
