use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::CreateEmbed;
use poise::CreateReply;

#[poise::command(prefix_command, slash_command, category = "role_util", owners_only)]
pub async fn role_show(
    _ctx: Context<'_>,
    #[description = "guild id"] guild_id: serenity::GuildId,
    #[description = "role id"] role_id: serenity::RoleId,
) -> Result<(), Error> {
    let ret = role_show0(_ctx.clone(), guild_id, role_id).await;
    if let Err(err) = ret {
        _ctx.say(format!("Error {:?}", err)).await?;
    }

    Ok(())
}

async fn role_show0(
    _ctx: Context<'_>,
    guild_id: serenity::GuildId,
    role_id: serenity::RoleId,
) -> Result<(), Error> {
    let guild = _ctx.http().get_guild(guild_id.into()).await?;
    let role = guild.roles.get(&role_id).ok_or("Role not found")?;

    let _m = _ctx
        .send(
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("Role Info")
                    .field("Name:", &role.name, false)
                    .field("Id:", &role.id.to_string(), false)
                    .field("Color:", &role.colour.hex(), false)
                    .field("Position", &role.position.to_string(), false)
                    .field("Permission", &role.permissions.to_string(), false),
            ),
        )
        .await?;

    Ok(())
}

#[poise::command(prefix_command, slash_command, category = "role_util", owners_only)]
pub async fn role_move(
    _ctx: Context<'_>,
    #[description = "guild id"] guild_id: serenity::GuildId,
    #[description = "role id"] role_id: serenity::RoleId,
    #[description = "position"] position: u64,
) -> Result<(), Error> {
    let guild = _ctx.http().get_guild(guild_id.into()).await?;
    let role = guild.roles.get(&role_id).ok_or("Role not found")?;

    let res = guild.edit_role_position(_ctx, role, position as u16).await;
    match res {
        Ok(_r) => {
            _ctx.say(format!("Set role:{} to pos:{}", role.name, position))
                .await?;
        }
        Err(err) => {
            _ctx.say(format!("Error: {:?}", err)).await?;
        }
    }

    Ok(())
}

/*
#[group]
#[commands(role_show, role_move)]
struct RoleUtil;

#[command]
#[owners_only]
#[usage = "<guild> <role>"]
async fn role_show(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let ret = role_show0(_ctx, _msg, _args).await;
    if let Err(err) = ret {
        _msg.reply(_ctx, format!("Error {:?}", err)).await?;
    }

    Ok(())
}

async fn role_show0(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = GuildId::from(_args.single::<u64>()?);
    let role_id = _args.single::<RoleId>()?;

    let guild = _ctx.http.get_guild(guild_id.into()).await?;
    let role = guild.roles.get(&role_id).ok_or("Role not found")?;

    let _m = _msg
        .channel_id
        .send_message(_ctx, |m| {
            m.embed(|e| {
                e.title("Role info");
                e.field("Name:", &role.name, false);
                e.field("Id:", &role.id.to_string(), false);
                e.field("Color:", &role.colour.hex(), false);
                e.field("Position:", &role.position, false);
                e.field("Permisson:", &role.permissions.to_string(), false);
                e
            });
            m.reference_message(_msg);
            m
        })
        .await?;

    Ok(())
}

#[command]
#[owners_only]
#[usage = "<guild> <role> <position>"]
async fn role_move(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    let guild_id = GuildId::from(_args.single::<u64>()?);
    let role_id = _args.single::<RoleId>()?;
    let position = _args.single::<u64>()?;

    let guild = _ctx.http.get_guild(guild_id.into()).await?;
    let role = guild.roles.get(&role_id).ok_or("Role not found")?;

    let res = guild.edit_role_position(_ctx, role, position).await;
    match res {
        Ok(_r) => {
            _msg.reply(&_ctx, format!("Set role:{} to pos:{}", role.name, position))
                .await?;
        }
        Err(err) => {
            _msg.reply(&_ctx, format!("Error: {:?}", err)).await?;
        }
    }

    Ok(())
}
*/
