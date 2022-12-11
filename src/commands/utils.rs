use serenity::prelude::*;
use serenity::{model::gateway::Activity, model::gateway::ActivityType};
use sqlx::Pool;
use sqlx::Any;

pub struct PgContainer;

impl TypeMapKey for PgContainer {
    type Value = Pool<Any>;
}

#[allow(dead_code)]
pub fn custom_activity<N>(message: N) -> Activity
where
    N: ToString,
{
    let mut act = Activity::playing(message.to_string());
    act.kind = ActivityType::Custom;
    act
}
