use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Guild not found in cache")]
    GuildNotFound,

    #[error("Role not found")]
    RoleNotFound,

    #[error("Message is not from guild")]
    MessageNotFromGuild,

    #[error("This role is not belong to this guild")]
    RoleNotInGuild,

    #[error("Cannot get cache")]
    CacheFail,
}
